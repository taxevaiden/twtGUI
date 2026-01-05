#ifndef CONFIG_H
#define CONFIG_H

#include <string>

#include "SimpleIni.h"

namespace twtgui
{
    class GlobalConfig
    {
        public:
            static void loadConfig(std::string configFile);
            static CSimpleIniA config;

    };
}

#endif