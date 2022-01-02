#include <QObject>
#include <QQmlApplicationEngine>
#include <QGuiApplication>
#include <QQmlContext>
#include <QThread>

#include "gui.h"

namespace {
    QString GuiStringToQString(const String& s) {
        return QString::fromUtf8(reinterpret_cast<const char*>(s.data), s.len);
    }

    String QStringToGuiString(const QString& s) {
        auto byte_arr = s.toUtf8();
        String guiString;
        guiString.data = reinterpret_cast<const uint8_t *>(byte_arr.data());
        guiString.len = byte_arr.size();
        return guiString;
    }
}

class Backend : public QObject
{
    Q_OBJECT

    Q_PROPERTY(QVariantList output MEMBER output_ NOTIFY OutputChanged)
    Q_PROPERTY(QVariantList voices MEMBER voices_ NOTIFY VoicesChanged)

public:
    Backend(GuiCallbacks callbacks, QVariantList voices, const void* data)
        : callbacks_(callbacks)
        , voices_(voices)
        , data_(data)
    {}

public slots:
    void PushOutput(const QString& text) {
        if (QThread::currentThread() != thread()) {
            QMetaObject::invokeMethod(this, [=] {
                PushOutput(text);
            });
            return;
        }

        output_.push_back(text);
        emit OutputChanged();
    }

    void PushLoopStart(
        const QString& text,
        const QString& voice,
        int32_t numIters)
    {
        if (QThread::currentThread() != thread()) {
            QMetaObject::invokeMethod(this, [=] {
                PushLoopStart(text, voice, numIters);
            });
            return;
        }

        auto output = tr("\nStarting loop...\n"
            "Text: %1\n"
            "Voice: %2\n"
            "Iterations: %3\n")
            .arg(text)
            .arg(voice)
            .arg(numIters);
        output_.push_back(output);
        emit OutputChanged();
    }

    void PushError(const QString& error) {
        if (QThread::currentThread() != thread()) {
            QMetaObject::invokeMethod(this, [=] {
                PushError(error);
            });
            return;
        }

        auto output = tr("Error: %1").arg(error);
        output_.push_back(output);
        emit OutputChanged();
    }

public slots:
    void RunLoop(const QString& text, int num_iters, bool play, const QString& voice) {
        callbacks_.start_tts_loop(QStringToGuiString(text), num_iters, play, QStringToGuiString(voice), data_);
    }

signals:
    void OutputChanged();
    void VoicesChanged();

private:
    GuiCallbacks callbacks_;
    QVariantList voices_;
    const void* data_;
    QVariantList output_;
};

struct Gui
{
    GuiCallbacks callbacks;
    QVariantList voices;
    Backend* backend = nullptr;

    Gui(GuiCallbacks callbacks, QVariantList voices)
        : callbacks(callbacks)
        , voices(voices)
    {}
};

Gui* MakeGui(GuiCallbacks callbacks, const String* voices, uint64_t num_voices) {
    QVariantList qVoices;
    for (uint64_t i = 0; i < num_voices; ++i) {
        qVoices.push_back(GuiStringToQString(voices[i]));
    }
    return new Gui(callbacks, qVoices);
}

void DestroyGui(Gui* gui) {
    delete gui;
}

void Exec(Gui* gui, const void* data) {
    Q_INIT_RESOURCE(res);
    int argc = 0;
    QGuiApplication app(argc, nullptr);

    Backend backend(gui->callbacks, gui->voices, data);
    gui->backend = &backend;

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("backend", gui->backend);
    engine.load(QUrl("qrc:/Main.qml"));

    app.exec();

    gui->backend = nullptr;
}

void PushLoopStart(Gui* gui, String text, String voice, int32_t num_iters) {
    if (gui->backend) {
        gui->backend->PushLoopStart(
            GuiStringToQString(text),
            GuiStringToQString(voice),
            num_iters);
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


#include "gui.moc"
