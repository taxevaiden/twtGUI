#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>

namespace twtgui {

class MainWindow : public QMainWindow
{
    Q_OBJECT

    public:
        MainWindow(QWidget *parent = nullptr);
        ~MainWindow();
        void showEvent(QShowEvent* ev);
    };

} // namespace twtgui

using MainWindow = twtgui::MainWindow;

#endif // MAINWINDOW_H
