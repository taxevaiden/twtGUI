#include "downloadtask.h"
#include "download.h"

#include <sstream>
#include <vector>
#include <QDebug>
#include <fstream>
#include <filesystem>

namespace twtgui
{
    void multilineExtension(std::string &content)
    {
        const std::string sep = u8"\u2028";
        const std::string rep = "<br />";

        size_t pos = 0;
        while ((pos = content.find(sep, pos)) != std::string::npos)
        {
            content.replace(pos, sep.size(), rep);
            pos += rep.size();
        }
    }

    void addLinkTags(std::string &content)
    {
        std::stringstream ss(content);
        std::vector<std::string> words;
        std::string word;

        std::string modifiedContent = "";
        while (ss >> word)
        {
            words.push_back(word);
        }

        for (const auto &w : words)
        {
            std::string modifiedWord = w;
            std::size_t found_pos = w.find("http://");
            if (found_pos != std::string::npos)
            {
                modifiedWord = "<a href='" + w + "'>" + w + "</a>";
            }
            found_pos = w.find("https://");
            if (found_pos != std::string::npos)
            {
                modifiedWord = "<a href='" + w + "'>" + w + "</a>";
            }

            modifiedContent += modifiedWord;
            modifiedContent += " ";
        }

        content = modifiedContent;
    }

    DownloadTask::DownloadTask(QString url, QString source, QObject *parent)
        : QObject(parent),
          m_url(std::move(url)),
          m_source(std::move(source))
    {
        setAutoDelete(true); // IMPORTANT
    }

    DownloadTask::~DownloadTask() = default;

    void DownloadTask::cancel()
    {
        m_cancelled.store(true, std::memory_order_relaxed);
    }

    void DownloadTask::run()
    {
        if (m_url.isEmpty())
        {
            emit status("No URL provided");
            emit finished();
            return;
        }

        // cache directory
        if (!std::filesystem::is_directory("cache"))
            std::filesystem::create_directory("cache");

        auto hostFromUrl = [](const std::string &u) -> std::string
        {
            if (u.empty())
                return {};
            size_t pos = u.find("://");
            size_t start = (pos == std::string::npos) ? 0 : pos + 3;
            size_t end = u.find('/', start);
            return u.substr(start, end == std::string::npos ? std::string::npos : end - start);
        };

        std::string outPath =
            "cache/" + hostFromUrl(m_url.toStdString()) + ".txt";

        TwtDownloader downloader;

        if (!std::filesystem::exists(outPath) ||
            downloader.remoteChanged(m_url.toStdString(), outPath))
        {
            emit status(QString("Downloading %1 ...").arg(m_url));

            TwtDownloader::Result result =
                downloader.downloadToFile(m_url.toStdString(),
                                          outPath,
                                          30,
                                          true);

            if (!result.success)
            {
                emit error(QString::fromStdString(result.error));
                emit finished();
                return;
            }

            emit status(QString("Downloaded %1. Parsing...").arg(m_url));
        }
        else
        {
            emit status(QString("Using cache for %1. Parsing ...").arg(m_url));
        }

        std::ifstream text(outPath);
        std::string line;

        while (!m_cancelled.load(std::memory_order_relaxed) &&
               std::getline(text, line))
        {
            if (!line.empty() && line.back() == '\r')
                line.pop_back();

            size_t tab = line.find('\t');
            if (tab == std::string::npos)
                continue;

            std::string timestamp = line.substr(0, tab);
            std::string content = line.substr(tab + 1);

            if (!timestamp.empty() && timestamp.back() == '\r')
                timestamp.pop_back();
            if (!content.empty() && content.back() == '\r')
                content.pop_back();

            addLinkTags(content);

            emit tweetReady(QString::fromStdString(timestamp),
                            QString::fromStdString(content),
                            m_source);
        }

        emit status(QString("Done parsing feed for %1").arg(m_url));

        emit finished();

        return;
    }
}
