#include "gui.h"
#include <QGuiApplication>
#include <QObject>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QStringListModel>
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

class Backend : public QObject {
  Q_OBJECT

  Q_PROPERTY(QStringListModel* output READ Output NOTIFY OutputChanged)
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

  void PushLoopStart(const QString& text, const QString& voice,
                     int32_t numIters) {
    if (QThread::currentThread() != thread()) {
      QMetaObject::invokeMethod(this,
                                [=] { PushLoopStart(text, voice, numIters); });
      return;
    }

    auto output = tr("<b>Starting loop. Voice: %1, Iterations: %2<br>"
                     "%3</b>")
                      .arg(voice.toHtmlEscaped())
                      .arg(numIters)
                      .arg(text.toHtmlEscaped());

    PushOutputRaw(output);
  }

  void PushError(const QString& error) {
    if (QThread::currentThread() != thread()) {
      QMetaObject::invokeMethod(this, [=] { PushError(error); });
      return;
    }

    auto output =
        tr("<b><span style=\"color:red\">Error: %1</span></b>").arg(error);
    PushOutputRaw(output);
  }

  void PushCancel() {
    if (QThread::currentThread() != thread()) {
      QMetaObject::invokeMethod(this, [this] { PushCancel(); });
      return;
    }

    auto output =
        tr("<b><span style=\"color:red\">Canceled</span></b>");
    PushOutputRaw(output);
  }

 public slots:
  void RunLoop(const QString& text, int num_iters) {
    callbacks_.start_tts_loop(QStringToGuiString(text).s, num_iters, data_);
  }

  void SetVoice(int voice_idx) {
    callbacks_.set_voice(QStringToGuiString(voices_[voice_idx]).s, data_);
  }

  void EnableAudio(bool enable) {
    callbacks_.enable_audio(enable, data_);
  }

  void Cancel() { callbacks_.cancel(data_); }

  QStringListModel* Output() { return &output_; }

 signals:
  void OutputChanged();
  void VoicesChanged();

 private:
  void PushOutputRaw(const QString& text) {
    output_.insertRow(0);
    auto index = output_.index(0, 0);
    output_.setData(index, text);
  }
  GuiCallbacks callbacks_;
  QStringList voices_;
  const void* data_;
  QStringListModel output_;
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

  Backend backend(gui->callbacks, gui->voices, data);
  gui->backend = &backend;

  QQmlApplicationEngine engine;
  engine.rootContext()->setContextProperty("backend", gui->backend);
  engine.load(QUrl("qrc:/Main.qml"));

  QGuiApplication::exec();

  gui->backend = nullptr;
}

void PushLoopStart(Gui* gui, String text, String voice, int32_t num_iters) {
  if (gui->backend) {
    gui->backend->PushLoopStart(GuiStringToQString(text),
                                GuiStringToQString(voice), num_iters);
  }
}

void PushOutput(Gui* gui, String text) {
  if (gui->backend) {
    gui->backend->PushOutput(GuiStringToQString(text));
  }
}

void PushError(Gui* gui, String error) {
  if (gui->backend) {
    gui->backend->PushError(GuiStringToQString(error));
  }
}

void PushCancel(Gui* gui) {
  if (gui->backend) {
    gui->backend->PushCancel();
  }
}

#include "gui.moc"
