#ifndef PANEL_H
#define PANEL_H

#include <QWidget>
#include <QPushButton>
#include <QVBoxLayout>
#include <QFormLayout>
#include <string>

#include "SimpleIni.h"

namespace twtgui {
    enum SettingType {
        SettingType_Check,
        SettingType_FilePath,
        SettingType_Text,
        SettingType_Number
    };

    class SettingsPanel : public QWidget
    {
        Q_OBJECT

        public:
            SettingsPanel(QWidget* parent = nullptr);
            ~SettingsPanel();

            void addSetting(std::string label = "", std::string tooltip = "", std::string key = "", SettingType type = SettingType_Check);
        private slots:
            void applySettings();
        private:
            QPushButton* applyButton;
            QVBoxLayout* mainLayout;
            QFormLayout* formLayout;
            QWidget *formWidget;
            CSimpleIniA config;
            std::map<std::string, std::string> settings;
    };
}

#endif // PANEL_H