#ifndef TIMELINE_H
#define TIMELINE_H

#include <QPushButton>
#include <QVBoxLayout>

class Timeline : public QWidget
{
    Q_OBJECT
    public:
        Timeline(QWidget *parent = nullptr, std::string configFile = "");
        ~Timeline();
        void refreshTimeline();
    private slots:
        void handleButtonClick();
    private:
        QPushButton* refreshButton;
        std::string configFile;
        QVBoxLayout* mainLayout;
        QVBoxLayout* tweetsLayout;
};

#endif // TIMELINE_H