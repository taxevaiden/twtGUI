#ifndef VIEWFEED_H
#define VIEWFEED_H

#include <string>
#include <QLabel>
#include <QPushButton>
#include <QVBoxLayout>
#include <QListView>
#include <QStandardItemModel>
#include <QThread>

#include <mutex>
#include <vector>
#include <tuple>

namespace twtgui {

class ViewFeed : public QWidget
{
    Q_OBJECT
    public:
        ViewFeed(QWidget *parent = nullptr, std::string configFile = "");
        ~ViewFeed();
        void refreshTimeline(std::string url = "");
    private slots:
        void handleButtonClick();
        void onWorkerTweet(const QString &timestamp, const QString &content, const QString &source);
        void onWorkerStatus(const QString &statusMsg);
        void onWorkerFinished();
    private:
        void stopWorker();

        QLabel* statusLabel;
        QPushButton* refreshButton;
        std::string configFile;
        QVBoxLayout* mainLayout;
        QListView* tweetsView;
        QStandardItemModel* tweetsModel;
        std::string lastUrl;

        // background worker
        QThread* workerThread = nullptr;
        QObject* workerObj = nullptr; // pointer to DownloadWorker (opaque here to avoid include in header)

        std::mutex workerMutex;
        std::vector<std::tuple<QDateTime, std::string, std::string>> collectedTweets;
};

} // namespace twtgui

using ViewFeed = twtgui::ViewFeed;

#endif // VIEWFEED_H