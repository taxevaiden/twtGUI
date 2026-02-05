#ifndef TIMELINE_H
#define TIMELINE_H

#include <string>
#include <QWidget>
#include <QLabel>
#include <QPushButton>
#include <QVBoxLayout>
#include <QListView>
#include <QStandardItemModel>
#include <QThread>
#include <QDateTime>
#include <QString>

#include <mutex>
#include <vector>

#include "SimpleIni.h"
#include "downloadtask.h"

namespace twtgui {

struct Tweet {
    std::string timestamp; // ISODate string
    std::string content;
    std::string source;
};

class Timeline : public QWidget
{
    Q_OBJECT

    public:
        Timeline(QWidget *parent = nullptr);
        ~Timeline();
        void addTweet(std::string timestamp, std::string content, std::string source = "");
        void refreshTimeline();
    signals:
        void allTweetsReady();
    private slots:
        void handleButtonClick();
        // slots called from background workers (queued connections)
        void onWorkerTweet(const QString &timestamp, const QString &content, const QString &source);
        void onWorkerStatus(const QString &statusMsg);
        void onWorkerFinished();
        void updateTweetsView();
    private:
        void stopWorkers();

        QLabel* statusLabel;
        QPushButton* refreshButton;
        CSimpleIniA config;
        std::string configFile;
        QVBoxLayout* mainLayout;
        QListView* tweetsView;
        QStandardItemModel* tweetsModel;

        // background worker tracking
        std::mutex workerMutex;
        std::vector<QThread*> workerThreads;
        std::vector<QObject*> workers;
        int pendingWorkers = 0;
        std::vector<Tweet> collectedTweets;

        std::vector<DownloadTask*> tasks;
};

} // namespace twtgui

using Timeline = twtgui::Timeline;

#endif // TIMELINE_H