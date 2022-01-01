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

    Rectangle {
        id: root
        anchors.fill: parent
        anchors.margins: 15

        Column {
            anchors.fill: parent
            spacing: 5

            RowLayout {
                id: inputLayout

                anchors.right: parent.right
                anchors.left: parent.left

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
                        Layout.leftMargin: 0
                        id: play
                        checkState: Qt.Unchecked
                    }

                    Text {
                        Layout.alignment: Qt.AlignRight
                        text: qsTr("Voice")
                    }

                    ComboBox {
                        Layout.alignment: Qt.AlignLeft
                        id: selectedVoice
                        model: backend.voices
                    }

                    Button {
                        Layout.alignment: Qt.AlignRight
                        text: qsTr("Run loop")

                        onClicked: {
                            backend.RunLoop(inputText.text, numIters.value, play.checkState, backend.voices[selectedVoice.currentIndex])
                        }
                    }

                    Button {
                        Layout.alignment: Qt.AlignLeft
                        text: qsTr("Cancel")

                        onClicked: {
                            backend.Cancel()
                        }
                    }

                }
            }

            Rectangle {
                anchors.right: parent.right
                anchors.left: parent.left

                height: root.height - inputLayout.height

                border.color: "lightgrey"

                ListView {
                    id: outputView

                    anchors.fill: parent
                    anchors.margins: 2

                    clip: true
                    verticalLayoutDirection: ListView.BottomToTop

                    model: backend.output
                    delegate: TextEdit {
                        id: outputText

                        text: display
                        textFormat: TextEdit.RichText

                        readOnly: true
                        selectByMouse: true
                    }

                    ScrollBar.vertical : ScrollBar {}
                }
            }
        }
    }
}
