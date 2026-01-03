#include "window.h"

#include <QDebug>
#include <QShowEvent>
#include <fstream>

namespace twtgui {

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
{
    qDebug() << "twtgui::MainWindow constructed";
    fprintf(stderr, "twtgui::MainWindow constructed\n");
    {
        std::ofstream log("C:/Users/aiden/AppData/Local/twtgui_startup.log", std::ios::app);
        if (log) log << "MainWindow constructed\n";
    }
}

void MainWindow::showEvent(QShowEvent* ev)
{
    QMainWindow::showEvent(ev);
    qDebug() << "twtgui::MainWindow showEvent - visible=" << isVisible();
    fprintf(stderr, "twtgui::MainWindow showEvent - visible=%d\n", static_cast<int>(isVisible()));
    {
        std::ofstream log("C:/Users/aiden/AppData/Local/twtgui_startup.log", std::ios::app);
        if (log) log << "MainWindow showEvent - visible=" << (isVisible() ? "1" : "0") << "\n";
    }
}

MainWindow::~MainWindow() {
    qDebug() << "twtgui::MainWindow destroyed";
}

} // namespace twtgui
