import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick 2.15

ApplicationWindow {
    title: qsTr("Tts looper")
    width: 640
    height: 480
    visible: true
    minimumWidth: 640
    minimumHeight: 480

    ColumnLayout {
        anchors.fill:parent 
        RowLayout {
            ColumnLayout {
                Text {
                    text: qsTr("Tts text")
                }
                
                TextField {
                    id: inputText
                    Layout.fillWidth: true
                    Layout.minimumHeight: 200
                    placeholderText: qsTr("Tts text")
                }
            }

            GridLayout {
                id: settings
                columns: 2

                Text {
                    Layout.alignment: Qt.AlignRight
                    text: qsTr("Number of iterations")
                }

                SpinBox {
                    Layout.alignment: Qt.AlignLeft
                    id: numIters
                    value: 10
                }

                Text {
                    Layout.alignment: Qt.AlignRight
                    text: qsTr("Play audio")
                }

                CheckBox {
                    Layout.alignment: Qt.AlignLeft
                    id: play
                    checkState: Qt.Unchecked
                }

                Button {
                    Layout.alignment: Qt.AlignCenter
                    text: qsTr("Run loop")

                    Layout.columnSpan: settings.columns

                    onClicked: {
                        backend.RunLoop(inputText.text, numIters.value, play.checkState)
                    }
                }

            }
        }

        TextEdit {
            id: outputText

            Layout.fillHeight: true

            text: backend.output
            readOnly: true
            wrapMode: Text.WordWrap
            selectByMouse: true
        }
    }
}
