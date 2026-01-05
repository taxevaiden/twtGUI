#include "config.h"

#include "SimpleIni.h"

namespace twtgui {
    CSimpleIniA GlobalConfig::config;
    
    void twtgui::GlobalConfig::loadConfig(std::string configFile) {
        config.LoadFile(configFile.c_str());
    }
}