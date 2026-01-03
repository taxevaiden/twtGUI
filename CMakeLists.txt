cmake_minimum_required(VERSION 3.16)

project(twtGUI VERSION 0.1 LANGUAGES CXX)

set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(QT NAMES Qt6 Qt5 REQUIRED COMPONENTS Widgets)
find_package(Qt${QT_VERSION_MAJOR} REQUIRED COMPONENTS Widgets)

file(GLOB MAIN_SOURCES
    src/*.cpp
    src/*.h
)

file(GLOB UI_SOURCES
    src/ui/*.cpp
    src/ui/*.h
    src/ui/*/*.cpp
    src/ui/*/*.h
)

file(GLOB CURL_LIBS
    ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/lib/*.a
)

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

target_link_libraries(twtGUI PRIVATE Qt${QT_VERSION_MAJOR}::Widgets)
target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/include)

target_include_directories(twtGUI PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/external/curl/include)
target_link_libraries(twtGUI PRIVATE ${CURL_LIBS})

# Qt for iOS sets MACOSX_BUNDLE_GUI_IDENTIFIER automatically since Qt 6.1.
# If you are developing for iOS or macOS you should consider setting an
# explicit, fixed bundle identifier manually though.
if(${QT_VERSION} VERSION_LESS 6.1.0)
  set(BUNDLE_ID_OPTION MACOSX_BUNDLE_GUI_IDENTIFIER com.example.twtGUI)
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

# Copy Qt platform plugins into the target runtime directory so the app finds platform plugins
# even when launched by the debugger from a different environment.
if(DEFINED Qt${QT_VERSION_MAJOR}_DIR)
    get_filename_component(_qt_cmake_dir "${Qt${QT_VERSION_MAJOR}_DIR}" ABSOLUTE)
    # Walk up from .../lib/cmake/Qt6 -> .../lib/cmake -> .../lib -> .../mingw_64 (root)
    get_filename_component(_qt_cmake_parent "${_qt_cmake_dir}" DIRECTORY)
    get_filename_component(_qt_lib_dir "${_qt_cmake_parent}" DIRECTORY)
    get_filename_component(_qt_root_dir "${_qt_lib_dir}" DIRECTORY)
    set(_qt_plugins_dir "${_qt_root_dir}/plugins")
    set(_qt_platforms_dir "${_qt_plugins_dir}/platforms")
    message(STATUS "Qt platforms dir detected: ${_qt_platforms_dir}")
    if(EXISTS "${_qt_platforms_dir}")
        add_custom_command(TARGET twtGUI POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_FILE_DIR:twtGUI>/platforms"
            COMMAND ${CMAKE_COMMAND} -E copy_directory "${_qt_platforms_dir}" "$<TARGET_FILE_DIR:twtGUI>/platforms"
        )
    else()
        message(WARNING "Qt platforms dir not found at ${_qt_platforms_dir}; platform plugins will not be copied.")
    endif()
else()
    message(WARNING "Qt${QT_VERSION_MAJOR}_DIR is not defined; cannot auto-copy Qt platform plugins.")
endif()

if(QT_VERSION_MAJOR EQUAL 6)
    qt_finalize_executable(twtGUI)
endif()
