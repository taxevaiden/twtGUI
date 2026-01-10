#include "downloadworker.h"
#include "download.h"

#include <sstream>
#include <vector>
#include <QDebug>
#include <fstream>

#include <filesystem>

namespace twtgui
{

    DownloadWorker::DownloadWorker(QObject *parent)
        : QObject(parent), m_cancelled(false)
    {
    }

    DownloadWorker::~DownloadWorker() = default;

    void DownloadWorker::cancel()
    {
        m_cancelled.store(true);
    }

    void DownloadWorker::start(const QString &url, const QString &source)
    {
        if (url.isEmpty())
        {
            emit status("No URL provided");
            emit finished();
            return;
        }

        if (!std::filesystem::is_directory("cache"))
            std::filesystem::create_directory("cache");

        auto hostFromUrl = [](const std::string &u) -> std::string
        {
            if (u.empty())
                return "";
            size_t pos = u.find("://");
            size_t start = (pos == std::string::npos) ? 0 : pos + 3;
            size_t end = u.find('/', start);
            return u.substr(start, end == std::string::npos ? std::string::npos : end - start);
        };

        std::string outPath = "cache/" + hostFromUrl(url.toStdString()) + ".txt";
        TwtDownloader downloader;

        if (!std::filesystem::exists(outPath) || downloader.remoteChanged(url.toStdString(), outPath))
        {
            emit status(QString("Downloading %1 ...").arg(url));

            TwtDownloader::Result result = downloader.downloadToFile(url.toStdString(), outPath, 30, true);
            if (!result.success)
            {
                emit error(QString::fromStdString(result.error));
                emit finished();
                return;
            }
            emit status(QString("Downloaded %1. Parsing...").arg(url));
        } else emit status(QString("Using cache for %1. Parsing ... ").arg(url));

        std::ifstream text(outPath);

        std::string line;

        while (!m_cancelled.load() && std::getline(text, line))
        {
            // trim potential CR
            if (!line.empty() && line.back() == '\r')
                line.pop_back();

            size_t tab = line.find('\t');
            if (tab == std::string::npos)
                continue;

            std::string timestamp = line.substr(0, tab);
            std::string content = line.substr(tab + 1);

            // trim trailing CR from timestamp/content
            if (!timestamp.empty() && timestamp.back() == '\r')
                timestamp.pop_back();
            if (!content.empty() && content.back() == '\r')
                content.pop_back();

            // qDebug() << "DownloadWorker parsed tweet:" << QString::fromStdString(timestamp) << "::" << QString::fromStdString(content);
            emit tweetReady(QString::fromStdString(timestamp), QString::fromStdString(content), source);
        }

        emit finished();
    }

} // namespace twtgui
