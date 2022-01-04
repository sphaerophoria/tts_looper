import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick 2.15
import Qt.labs.platform 1.1



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
                        text: qsTr("TTS text")
                        font.bold: true
                    }

                    ScrollView {
                        Layout.fillWidth: true
                        Layout.minimumHeight: 200
                        Layout.maximumHeight: settings.height

                        Rectangle {
                            border.color: "lightgrey"
                            anchors.fill: parent

                            TextArea {
                                id: inputText

                                anchors.fill: parent
                                anchors.margins: 3

                                Component.onCompleted: {
                                    backend.InputText.connect(updateText)

                                }

                                function updateText(s) {
                                    inputText.text = s
                                }

                                verticalAlignment: TextInput.AlignTop
                                placeholderText: qsTr("Tts text")
                                wrapMode: TextInput.Wrap
                            }
                        }
                    }
                }

                ColumnLayout {
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

                            onCheckStateChanged: {
                                backend.EnableAudio(checkState)
                            }
                        }

                        Text {
                            Layout.alignment: Qt.AlignRight
                            text: qsTr("Voice")
                        }

                        ComboBox {
                            Layout.alignment: Qt.AlignLeft
                            id: selectedVoice
                            model: backend.voices

                            onCurrentIndexChanged: {
                                backend.SetVoice(currentIndex)
                            }
                        }

                    }

                    RowLayout {
                        Button {
                            text: qsTr("Cancel")

                            onClicked: {
                                backend.Cancel()
                            }
                        }

                        Button {
                            text: qsTr("Run loop")

                            onClicked: {
                                backend.RunLoop(inputText.text, numIters.value)
                            }
                        }

                        Button {
                            text: qsTr("Save")

                            onClicked: {
                                fileDialog.file = ""
                                fileDialog.open()
                            }

                            FileDialog {
                                id: fileDialog
                                file: ""
                                folder: StandardPaths.writableLocation(StandardPaths.DocumentsLocation)
                                fileMode: FileDialog.SaveFile
                                nameFilters: [ "Wav files (*.wav)"]

                                onVisibleChanged: {
                                    if (file != "") {
                                        backend.Save(file)
                                    }
                                }
                            }
                        }
                    }

                    RowLayout {
                        Button {
                            property bool recording: false
                            text: recording ? qsTr("End recording") :  qsTr("Record input")

                            onClicked: {
                                if (recording) {
                                    backend.EndRecording()
                                } else {
                                    backend.StartRecording()
                                }
                                recording = !recording
                            }
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

                    function updateSelectionEnd() {
                        var yVal = mouseArea.mouseY + contentY
                        var idx = outputView.indexAt(0, yVal);
                        outputView.model.setSelectionEnd(idx)
                    }

                    clip: true
                    verticalLayoutDirection: ListView.BottomToTop

                    model: backend.output
                    delegate: Rectangle {
                        color: selected ? "lightsteelblue" : "white"

                        height: outputText.height
                        width: outputView.width

                        Text {
                            id: outputText

                            text: display
                            textFormat: TextEdit.RichText
                        }
                    }

                    onContentYChanged: {
                        if (mouseArea.pressed) {
                            updateSelectionEnd()
                        }

                    }

                    MouseArea {
                        id: mouseArea

                        anchors.fill: parent

                        preventStealing: true
                        propagateComposedEvents: false


                        onPressed: {
                            var contentY = mouseY + outputView.contentY
                            var idx = outputView.indexAt(0, contentY);
                            outputView.model.setSelectionStart(idx)
                        }

                        onPositionChanged: {
                            outputView.updateSelectionEnd()
                        }
                    }

                    Shortcut {
                        sequence: StandardKey.Copy
                        onActivated: backend.Copy()
                    }

                    ScrollBar.vertical : ScrollBar {}
                }
            }
        }
    }
}
