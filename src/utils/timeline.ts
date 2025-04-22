//what an entry SHOULD look like..
interface TimelineEntry {
  user: string;
  time: string;
  message: string;
}

//fetch timeline with the Cool api endpoint i made :3
async function fetchTimeline(
  url: string = "http://localhost:8080/timeline.txt",
): Promise<string> {
  try {
    const response = await fetch(url);

    if (!response.ok) {
      console.error(
        `Failed to fetch timeline. Status: ${response.status} ${response.statusText}`,
      );
      return "Failed to fetch timeline.";
    }

    const timeline = await response.text();
    return timeline;
  } catch (error) {
    console.error("Error fetching timeline:", error);
    return "Failed to fetch timeline due to an error.";
  }
}

function isImageLink(url: string) {
  const imageExtensions = [
    ".png",
    ".jpg",
    ".jpeg",
    ".gif",
    ".webp",
    ".bmp",
    ".svg",
  ];

  try {
    // Parse the URL to extract the pathname
    const pathname = new URL(url).pathname;

    // Check if the pathname ends with one of the image extensions
    return imageExtensions.some((ext) => pathname.toLowerCase().endsWith(ext));
  } catch (error) {
    console.error("Invalid URL:", error);
    return false;
  }
}

//format the message so that links automatically turn into <a> elements
function formatLinks(message: string): string {
  const images: string[] = [];
  // Regular expression to match URLs
  const urlRegex = /https?:\/\/[^\s]+/g;

  // Replace the matched URLs with anchor tags
  let formattedMessage = message.replace(urlRegex, (url) => {
    const isImage = isImageLink(url);

    if (isImage) {
      images.push(
        `<img class="mt-4 border border-green-200 shadow-glow block" src="${url}" alt="image from ${url}">`,
      );
    }

    return `<a href="${url}" target="_blank">${url}</a>`;
  });

  formattedMessage = "<p>&ThickSpace;" + formattedMessage + "</p>"; //put it all into a <p> element so uh styling works
  // add the images after the <p> element
  if (images.length > 0) {
    formattedMessage += images.join("");
  }

  return formattedMessage;
}

//format the timeline, turns it into a nice format just like TimelineEntry
function formatTimeline(timeline: string): TimelineEntry[] {
  // replace `âž¤` with `➤` (formatting is So Kind!)
  const cleanedTimeline = timeline.replace(/âž¤/g, "➤");

  // split entries based on `➤`
  const entries = cleanedTimeline.split("➤").slice(1); // Ignore the first empty split

  // format each entry
  const formattedEntries = entries
    .map((entry): TimelineEntry | null => {
      //tbh idek what this code does i just asked someone to help me and ctrl+c ctrl+v :sob:
      //I'm never getting a job

      // match the pattern with user, time, and message,, and handle multiline messages.
      const match = entry.match(/^(.*?)\s\((.*?)\):\s([\s\S]+)/); // match user, time, and message

      if (match) {
        const [, user, time, message] = match;

        const formattedMessage = formatLinks(message);

        return {
          user: user.trim(),
          time: time.trim(),
          message: formattedMessage.trim(),
        };
      }

      return null; // Skip invalid entries
    })
    .filter((entry): entry is TimelineEntry => entry !== null); // filter out null entries

  return formattedEntries.length > 0
    ? formattedEntries
    : [
        {
          user: "error!",
          time: "",
          message: "no valid entries found. ig you haven't posted anything",
        },
      ];
}
export { fetchTimeline, formatTimeline };
