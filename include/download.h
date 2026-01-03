#ifndef TWTGUI_DOWNLOAD_H
#define TWTGUI_DOWNLOAD_H

#include <string>
#include <cstddef>
#include <mutex>

// - downloadToFile: downloads the URL to a local file path
// - downloadToString: downloads the URL into a std::string
namespace twtgui {

class TwtDownloader {
public:
    struct Result {
        bool success = false;
        long http_code = 0;
        std::string error;
        std::size_t bytesDownloaded = 0;
    };

    TwtDownloader();
    ~TwtDownloader();

    Result downloadToFile(const std::string& url, const std::string& outputPath, long timeoutSeconds = 0, bool verifyPeer = true);

    Result downloadToString(const std::string& url, std::string& outData, long timeoutSeconds = 0, bool verifyPeer = true);

    bool downloadToFileSimple(const std::string& url, const std::string& outputPath, std::string& outError, long timeoutSeconds = 0, bool verifyPeer = true);

private:
    static void ensureCurlInit();
    static std::once_flag s_curlInitFlag;
};

} // namespace twtgui

#endif // TWTGUI_DOWNLOAD_H
