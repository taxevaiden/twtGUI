#ifndef DOWNLOADTASK_H
#define DOWNLOADTASK_H

#include <QObject>
#include <QRunnable>
#include <atomic>

namespace twtgui
{
    class DownloadTask : public QObject, public QRunnable
    {
        Q_OBJECT

    public:
        explicit DownloadTask(QString url, QString source, QObject *parent = nullptr);
        ~DownloadTask() override;

        void run() override;

    public slots:
        void cancel();

    signals:
        void tweetReady(const QString &timestamp,
                        const QString &content,
                        const QString &source);
        void status(const QString &);
        void error(const QString &);
        void finished();

    private:
        QString m_url;
        QString m_source;
        std::atomic_bool m_cancelled{false};
    };
}

#endif