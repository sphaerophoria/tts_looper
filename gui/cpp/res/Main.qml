import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick 2.15

ApplicationWindow {
    //title of the application
    title: qsTr("Tts looper")
    width: 640
    height: 480
    visible: true
    minimumWidth: 640
    minimumHeight: 480

    //Content Area

    GridLayout {
        id: grid
        columns: 4
        anchors.fill: parent

        //a button in the middle of the content area
        TextField {
            id: inputText
            Layout.fillWidth: true
            placeholderText: qsTr("Tts text")
        }

        SpinBox {
            id: numIters
            value: 10
        }

        CheckBox {
            id: play
            text: qsTr("Play audio")
            checkState: Qt.Unchecked
        }

        Button {
            text: qsTr("Run loop")

            onClicked: {
                backend.RunLoop(inputText.text, numIters.value, play.checkState)
            }
        }

        TextEdit {
            id: outputText

            Layout.fillHeight: true
            Layout.columnSpan: grid.columns

            text: backend.output
            readOnly: true
            wrapMode: Text.WordWrap
            selectByMouse: true
        }
    }
}
