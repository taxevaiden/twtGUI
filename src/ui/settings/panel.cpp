#include "panel.h"

#include <QCheckBox>
#include <QLineEdit>
#include <QSpinBox>
#include <QFileDialog>
#include <string>

#include <map>

#include "../../config.h"


namespace twtgui {

    twtgui::SettingsPanel::SettingsPanel(QWidget* parent)
        : QWidget(parent)
    {
        this->config.LoadFile("config.ini");
        mainLayout = new QVBoxLayout(this);
        formWidget = new QWidget(this);
        formLayout = new QFormLayout(formWidget);
        applyButton = new QPushButton("Apply", this);
        formLayout->addWidget(applyButton);
        formWidget->setLayout(formLayout);
        mainLayout->addWidget(formWidget);
        mainLayout->addWidget(applyButton);
        setLayout(mainLayout);

        connect(applyButton, &QPushButton::clicked, this, &SettingsPanel::applySettings);
    }

    twtgui::SettingsPanel::~SettingsPanel() = default;

    void twtgui::SettingsPanel::applySettings()
    {
        for (const auto& [key, value] : settings) {
            twtgui::GlobalConfig::config.SetValue("settings", key.c_str(), value.c_str());
        }
        twtgui::GlobalConfig::config.SaveFile("config.ini");
    }

    void twtgui::SettingsPanel::addSetting(std::string label, std::string tooltip, std::string key, SettingType type)
    {
        QHBoxLayout* inputLayout = new QHBoxLayout();

        const char* value = twtgui::GlobalConfig::config.GetValue("settings", key.c_str(), "");

        switch (type) {
            case SettingType_Check: {
                QCheckBox* checkBox = new QCheckBox(this);
                checkBox->setChecked(std::string(value) == "1");

                if (!tooltip.empty()) {
                    checkBox->setToolTip(QString::fromStdString(tooltip));
                }

                connect(checkBox, &QCheckBox::checkStateChanged, [this, key](int state) {
                    settings[key] = state == Qt::Checked ? "1" : "0";
                });

                inputLayout->addWidget(checkBox);
                break;
            }
            case SettingType_FilePath: {
                QLineEdit* lineEdit = new QLineEdit(QString::fromStdString(value), this);
                QPushButton* browseButton = new QPushButton("Browse", this);

                if (!tooltip.empty()) {
                    lineEdit->setToolTip(QString::fromStdString(tooltip));
                }

                connect(browseButton, &QPushButton::clicked, [this, lineEdit, key]() {
                    QString filePath = QFileDialog::getOpenFileName(this, "Select File", "", "All Files (*)");
                    if (!filePath.isEmpty()) {
                        lineEdit->setText(filePath);
                        settings[key] = filePath.toStdString();
                    }
                });

                inputLayout->addWidget(lineEdit);
                inputLayout->addWidget(browseButton);
                break;
            }
            case SettingType_Text: {
                QLineEdit* lineEdit = new QLineEdit(QString::fromStdString(value), this);

                if (!tooltip.empty()) {
                    lineEdit->setToolTip(QString::fromStdString(tooltip));
                }

                connect(lineEdit, &QLineEdit::textChanged, [this, key](const QString& text) {
                    settings[key] = text.toStdString();
                });

                inputLayout->addWidget(lineEdit);
                break;
            }
            case SettingType_Number: {
                QSpinBox* spinBox = new QSpinBox(this);
                spinBox->setRange(INT_MIN, INT_MAX);

                bool ok = false;
                int v = QString::fromUtf8(value).toInt(&ok);
                spinBox->setValue(ok ? v : 0);

                if (!tooltip.empty()) {
                    spinBox->setToolTip(QString::fromStdString(tooltip));
                }

                connect(spinBox, qOverload<int>(&QSpinBox::valueChanged), [this, key](int val) {
                    settings[key] = std::to_string(val);
                });

                inputLayout->addWidget(spinBox);
                break;
            }
        }

        if (inputLayout) {
            formLayout->addRow(QString::fromStdString(label), inputLayout);
        }
    }

} // namespace twtgui