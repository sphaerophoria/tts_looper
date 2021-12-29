#include <QObject>
#include <QQmlApplicationEngine>
#include <QGuiApplication>
#include <QQmlContext>
#include <QThread>

#include "gui.h"

class Backend : public QObject
{
    Q_OBJECT

    Q_PROPERTY(QString output MEMBER output_ NOTIFY OutputChanged)

public:
    Backend(GuiCallbacks callbacks)
        : callbacks_(callbacks)
    {}

public slots:
    void PushOutput(const QString& text) {
        if (QThread::currentThread() != thread()) {
            QMetaObject::invokeMethod(this, [this, text] {
                PushOutput(text);
            });
            return;
        }
        if (!output_.isEmpty()) {
            output_.push_back('\n');
        }

        output_.push_back(text);
        emit OutputChanged();
    }

    void ResetOutput() {
        if (QThread::currentThread() != thread()) {
            QMetaObject::invokeMethod(this, [this] {
                ResetOutput();
            });
            return;
        }
        output_.clear();
    }

public slots:
    void RunLoop(const QString& text, int num_iters, bool play) {
        auto byte_arr = text.toUtf8();
        callbacks_.start_tts_loop(
            reinterpret_cast<const uint8_t *>(byte_arr.data()), byte_arr.size(),
            num_iters, play, callbacks_.data);
    }

signals:
    void OutputChanged();

private:
    GuiCallbacks callbacks_;
    QString output_;
};

struct Gui
{
    Backend backend;

    Gui(GuiCallbacks callbacks)
        : backend(callbacks)
    {}
};

Gui* MakeGui(GuiCallbacks callbacks) {
    return new Gui(callbacks);
}

void DestroyGui(Gui* gui) {
    delete gui;
}

void Exec(Gui* gui) {
    Q_INIT_RESOURCE(res);
    int argc = 0;
    QGuiApplication app(argc, nullptr);
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("backend", &gui->backend);
    engine.load(QUrl("qrc:/Main.qml"));
    app.exec();
}

void PushOutput(Gui* gui, const uint8_t* text, uint64_t text_len) {
    gui->backend.PushOutput(QString::fromUtf8(reinterpret_cast<const char*>(text), text_len));
}

void ResetOutput(Gui* gui) {
    gui->backend.ResetOutput();
}



#include "gui.moc"
