#ifndef TWTGUI_DOWNLOADWORKER_H
#define TWTGUI_DOWNLOADWORKER_H

#include <QObject>
#include <atomic>
#include <QString>

namespace twtgui {

class DownloadWorker : public QObject
{
    Q_OBJECT
public:
    explicit DownloadWorker(QObject *parent = nullptr);
    ~DownloadWorker();
public slots:
    void start(const QString &url, const QString &source = QString());
    void cancel();
signals:
    void tweetReady(const QString &timestamp, const QString &content, const QString &source);
    void status(const QString &statusMsg);
    void finished();
    void error(const QString &err);
private:
    std::atomic<bool> m_cancelled;
};

} // namespace twtgui

#endif // TWTGUI_DOWNLOADWORKER_H
