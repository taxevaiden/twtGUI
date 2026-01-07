cmake_minimum_required(VERSION 3.16)

project(twtGUI VERSION 0.1 LANGUAGES CXX)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(QT NAMES Qt6 REQUIRED COMPONENTS Core)
find_package(QT NAMES Qt6 REQUIRED COMPONENTS Widgets)
find_package(Qt${QT_VERSION_MAJOR} REQUIRED COMPONENTS Core)
find_package(Qt${QT_VERSION_MAJOR} REQUIRED COMPONENTS Widgets)

find_package(CURL REQUIRED)

file(GLOB MAIN_SOURCES
    src/*.cpp
    src/*.h
    "include/*.h"
)

file(GLOB UI_SOURCES
    src/ui/*.cpp
    src/ui/*.h
    src/ui/*/*.cpp
    src/ui/*/*.h
)

# file(GLOB CURL_LIBS
#     ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/lib/*.a
# )

set(PROJECT_SOURCES
    ${MAIN_SOURCES}
    ${UI_SOURCES}
)

if(${QT_VERSION_MAJOR} GREATER_EQUAL 6)
    qt_add_executable(twtGUI
        MANUAL_FINALIZATION
        ${PROJECT_SOURCES}
    )
endif()

if(MINGW)
    target_link_options(${PROJECT_NAME} PRIVATE "-static" "-static-libgcc" "-static-libstdc++")
elseif(UNIX AND NOT APPLE)
    target_link_options(${PROJECT_NAME} PRIVATE "-static-libgcc" "-static-libstdc++")
endif()

target_link_libraries(twtGUI PRIVATE Qt${QT_VERSION_MAJOR}::Core)
target_link_libraries(twtGUI PRIVATE Qt${QT_VERSION_MAJOR}::Widgets)
target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)

target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/include)
target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/include/brotli)
target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/include/curl)
target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/include/nghttp2)
target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/include/nghttp3)
target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/include/ngtcp2)
target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/include/openssl)
# target_link_libraries(twtGUI PRIVATE ${CURL_LIBS})

# Qt for iOS sets MACOSX_BUNDLE_GUI_IDENTIFIER automatically since Qt 6.1.
# If you are developing for iOS or macOS you should consider setting an
# explicit, fixed bundle identifier manually though.
if(${QT_VERSION} VERSION_LESS 6.1.0)
  set(BUNDLE_ID_OPTION MACOSX_BUNDLE_GUI_IDENTIFIER com.taxevaiden.twtGUI)
endif()
set_target_properties(twtGUI PROPERTIES
    ${BUNDLE_ID_OPTION}
    MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
    MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}
    MACOSX_BUNDLE TRUE
    WIN32_EXECUTABLE TRUE
)

include(GNUInstallDirs)
install(TARGETS twtGUI
    BUNDLE DESTINATION .
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)