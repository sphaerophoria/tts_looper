#include "gui.h"
#include <qabstractitemmodel.h>
#include <qobjectdefs.h>
#include <QClipboard>
#include <QGuiApplication>
#include <QMimeData>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
#include <QStringListModel>
#include <QTextDocumentFragment>
#include <QThread>

namespace {
QString GuiStringToQString(const String& s) {
  return QString::fromUtf8(reinterpret_cast<const char*>(s.data), s.len);
}

struct GuiStringData {
  QByteArray data;
  String s;
};

GuiStringData QStringToGuiString(const QString& s) {
  GuiStringData ret;
  ret.data = s.toUtf8();
  ret.s.data = reinterpret_cast<const uint8_t*>(ret.data.data());
  ret.s.len = ret.data.size();
  return ret;
}
}  // namespace

class OutputModel : public QAbstractListModel {
  Q_OBJECT

 public:
  int rowCount(const QModelIndex& parent) const override {
    return data_.size();
  }

  QVariant data(const QModelIndex& index, int role) const override {
    if (role == Qt::DisplayRole) {
      return data_[index.row()];
    }

    if (role == Qt::UserRole) {
      auto row = index.row();
      auto low = std::min(selection_start_, selection_end_);
      auto high = std::max(selection_start_, selection_end_);

      if (low < 0 || high < 0) {
        return false;
      }

      return row >= low && row <= high;
    }

    return QVariant();
  }

  QHash<int, QByteArray> roleNames() const override {
    return {{Qt::DisplayRole, "display"}, {Qt::UserRole, "selected"}};
  }

  void addOutput(const QString& output) {
    emit beginInsertRows(QModelIndex(), 0, 0);
    data_.push_front(output);
    if (selection_start_ >= 0) {
      selection_start_ += 1;
    }

    if (selection_end_ >= 0) {
      selection_end_ += 1;
    }

    emit endInsertRows();
  }

 public slots:
  void setSelectionStart(int start) {
    if (start < 0) {
      return;
    }

    auto old_start = selection_start_;
    auto old_end = selection_end_;

    selection_start_ = start;
    selection_end_ = -1;

    std::array<int, 3> vals{{old_start, old_end, start}};

    auto tl = index(*std::min_element(vals.begin(), vals.end()));
    auto br = index(*std::max_element(vals.begin(), vals.end()));
    if (tl.row() >= 0 && br.row() >= 0) {
      emit dataChanged(tl, br);
    }
  }

  void setSelectionEnd(int end) {
    if (end < 0) {
      return;
    }

    std::array<int, 3> vals{{selection_end_, selection_start_, end}};
    selection_end_ = end;

    auto tl = index(*std::min_element(vals.begin(), vals.end()));
    auto br = index(*std::max_element(vals.begin(), vals.end()));
    if (tl.row() >= 0 && br.row() >= 0) {
      emit dataChanged(tl, br);
    }
  }

  QString selectionString() {
    if (selection_start_ < 0 || selection_end_ < 0) {
      return QString();
    }

    QString ret;

    auto low = std::min(selection_start_, selection_end_);
    auto high = std::max(selection_start_, selection_end_);

    bool first_iter = true;
    // Laid out backwards, so iterate backwards
    for (int i = high; i >= low; --i) {
      if (!first_iter) {
        ret.push_back("<br>");
      } else {
        first_iter = false;
      }

      ret.push_back(data_[i]);
    }

    return ret;
  }

 private:
  QStringList data_;
  int selection_start_ = -1;
  int selection_end_ = -1;
};

class Backend : public QObject {
  Q_OBJECT

  Q_PROPERTY(QAbstractItemModel* output READ Output NOTIFY OutputChanged)
  Q_PROPERTY(QStringList voices MEMBER voices_ NOTIFY VoicesChanged)

 public:
  Backend(GuiCallbacks callbacks, QStringList voices, const void* data)
      : callbacks_(callbacks), voices_(std::move(voices)), data_(data) {}

 public slots:
  void PushOutput(const QString& text) {
    if (QThread::currentThread() != thread()) {
      QMetaObject::invokeMethod(this, [=] { PushOutput(text); });
      return;
    }

    PushOutputRaw(text.toHtmlEscaped());
  }

  void PushInputText(const QString& text) {
    if (QThread::currentThread() != thread()) {
      QMetaObject::invokeMethod(this, [=] { PushInputText(text); });
      return;
    }

    emit InputText(text);
  }

  void PushOutputRaw(const QString& text) {
    if (QThread::currentThread() != thread()) {
      QMetaObject::invokeMethod(this,
                                [=] { PushOutputRaw(text); });
      return;
    }

    output_.addOutput(text);
  }

 public slots:
  void RunLoop(const QString& text, int num_iters) {
    callbacks_.start_tts_loop(QStringToGuiString(text).s, num_iters, data_);
  }

  void SetVoice(int voice_idx) {
    callbacks_.set_voice(QStringToGuiString(voices_[voice_idx]).s, data_);
  }

  void EnableAudio(bool enable) { callbacks_.enable_audio(enable, data_); }

  void Cancel() { callbacks_.cancel(data_); }

  void Copy() {
    auto* clipboard = QGuiApplication::clipboard();
    auto rich_text = output_.selectionString();
    auto* rich_text_mime = new QMimeData();
    rich_text_mime->setHtml(rich_text);
    rich_text_mime->setText(
        QTextDocumentFragment::fromHtml(rich_text).toPlainText());
    clipboard->setMimeData(rich_text_mime);
  }

  void Save(const QUrl& path) {
    callbacks_.save(QStringToGuiString(path.toLocalFile()).s, data_);
  }

  void StartRecording() {
    callbacks_.start_recording(data_);
  }

  void EndRecording() {
    callbacks_.end_recording(data_);
  }

  QAbstractItemModel* Output() { return &output_; }

 signals:
  void OutputChanged();
  void VoicesChanged();
  void InputText(QString text);

 private:
  GuiCallbacks callbacks_;
  QStringList voices_;
  const void* data_;
  OutputModel output_;
};

struct Gui {
  GuiCallbacks callbacks;
  QStringList voices;
  Backend* backend = nullptr;

  Gui(GuiCallbacks callbacks, QStringList voices)
      : callbacks(callbacks), voices(std::move(voices)) {}
};

Gui* MakeGui(GuiCallbacks callbacks, const String* voices,
             uint64_t num_voices) {
  QStringList q_voices;
  for (uint64_t i = 0; i < num_voices; ++i) {
    q_voices.push_back(GuiStringToQString(voices[i]));
  }
  return new Gui(callbacks, q_voices);
}

void DestroyGui(Gui* gui) { delete gui; }

void Exec(Gui* gui, const void* data) {
  Q_INIT_RESOURCE(res);
  int argc = 0;
  QGuiApplication app(argc, nullptr);

  QQuickStyle::setStyle("Fusion");

  Backend backend(gui->callbacks, gui->voices, data);
  gui->backend = &backend;

  QQmlApplicationEngine engine;
  engine.rootContext()->setContextProperty("backend", gui->backend);
  engine.load(QUrl("qrc:/Main.qml"));

  QGuiApplication::exec();

  gui->backend = nullptr;
}

void PushOutput(Gui* gui, String text) {
  if (gui->backend) {
    gui->backend->PushOutput(GuiStringToQString(text));
  }
}

void PushInputText(Gui* gui, String text) {
  if (gui->backend) {
    gui->backend->PushInputText(GuiStringToQString(text));
  }

}

void PushRawOutput(Gui* gui, String text) {
  if (gui->backend) {
    gui->backend->PushOutputRaw(GuiStringToQString(text));
  }
}

#include "gui.moc"
