import time
import argparse
import requests
import json
from bs4 import BeautifulSoup as BS
from dataclasses import dataclass
from typing import List, Dict, Set
import re
import os

LINUX_UNPLUGGED_URL = "https://linuxunplugged.com"
EPISODES_BY_ID_FILENAME = "episodes_by_id_index.json"
EPISODES_BY_TAG_FILENAME = "episodes_by_tag_index.json"
TAGS_BY_TAG = "tags_by_tag_index.json"


@dataclass
class Episode:
    id: int
    title: str
    date: str
    duration: str  # metadata
    tags: List[str]
    url: str

    def toJSON(self) -> str:
        return json.dumps(self.__dict__, indent=2, sort_keys=True)


Episodes = List[Episode]


def filter_empty_items(items: List[str]) -> List[str]:
    return list(
        filter(
            lambda x: len(x) > 0,
            [s.replace("\n", "").strip() for s in items]
        )
    )


def parse_date(date: str) -> str:
    return date.replace("|", "").strip()


EPISODE_ID_PATTERN = re.compile(r"(\d+\:)")


def parse_id_from_title(title: str) -> str:
    re.findall(EPISODE_ID_PATTERN, title)
    match = re.search(EPISODE_ID_PATTERN, title)
    id = match.group(0)
    id = id.replace(":", "")
    return id


def get_page_content(page_id: int) -> str:
    url = f"{LINUX_UNPLUGGED_URL}/page/{page_id}"

    headers = {
        "User-Agent": "Chrome/103.0.0.0 Safari/537.36",
        "Accept": "*/*",
    }

    response = requests.get(url, headers=headers)

    # raise exception if there's an error
    response.raise_for_status()
    return response.text


def page_content_to_episodes(content: str) -> Episodes:
    soup = BS(content, "html.parser")
    episode_selector = "list-item prose"
    html_episode_list = soup.find_all("div", class_=episode_selector)
    episodes: Episodes = []

    for html_episode in html_episode_list:
        # href
        h3 = html_episode.find("h3")
        a_tag = h3.find("a")
        episode_url = f"{LINUX_UNPLUGGED_URL}{a_tag['href']}"
        title: str = h3.text.strip()

        episode_id = int(parse_id_from_title(title))

        # parent of date and duration
        span_elements = html_episode.find_all("span")

        # get date
        times = filter_empty_items([d.text for d in span_elements[0]]) \
            if len(span_elements) > 0 else []

        date = parse_date(times[0]) if len(times) > 0 else ""
        duration = times[1] if len(times) > 1 else ""

        # get tags
        tags = filter_empty_items(span_elements[1].text.split(
            ",")) if len(span_elements) > 1 else []

        episodes.append(
            Episode(episode_id, title, date, duration, tags, episode_url))

    return episodes


def download_pages(start: int, end: int):
    if not os.path.isdir("pages/"):
        os.mkdir("pages/")

    for i in range(start, end + 1):
        content = get_page_content(i)

        assert type(content) == str

        with open(f"pages/page_{i}.html", "w") as f:
            f.write(content)


def get_all_episodes() -> Episodes:
    all_episodes: Episodes = []

    for file in [file for file in os.listdir("pages/") if file.endswith(".html")]:
        print(f"Parsing page \"{file}\" to episodes")

        with open(f"pages/{file}", "r") as f:
            content = f.read()

        episodes = page_content_to_episodes(content)
        all_episodes.extend(episodes)

    return all_episodes


def index_episodes(all_episodes: Episodes) -> None:
    episodes_by_id: Dict[int, Episode] = dict()
    # Map<tag, episode_id>
    episodes_by_tag: Dict[str, List[int]] = dict()
    all_tags: Set[str] = set()
    tags_to_list: Dict[str, List[str]] = dict()

    for episode in all_episodes:
        # map id episode to episode
        episodes_by_id[episode.id] = episode

        for tag in set(episode.tags):
            # map tag to episode
            tag = tag.lower().strip()

            episode_ids = episodes_by_tag.get(tag, [])
            episode_ids.append(episode.id)
            episodes_by_tag[tag] = episode_ids

            all_tags.add(tag)

    for tag in all_tags:
        # map tag to similar tags
        tags_to_list[tag] = [x for x in all_tags if x.find(tag) != -1]

    # save json data
    episodes_by_id = {k: ep.__dict__ for k, ep in episodes_by_id.items()}

    write_file(EPISODES_BY_ID_FILENAME, json.dumps(
        episodes_by_id, indent=2, sort_keys=True))
    write_file(EPISODES_BY_TAG_FILENAME,
               json.dumps(episodes_by_tag, indent=2, sort_keys=True))
    write_file(TAGS_BY_TAG, json.dumps(tags_to_list, indent=2, sort_keys=True))


def write_file(path_to_file: str, content: str):
    with open(path_to_file, "w") as f:
        f.write(content)


def main():
    parser = argparse.ArgumentParser()

    # The script uses this to know the range of pages to request
    parser.add_argument(
        "--range",
        default=[1, 1],
        type=int,
        nargs=2,
        help="Range of pages to parse, [s,e] where s and e are integers and e is inclusive"
    )

    parser.add_argument(
        "--download",
        default=False,
        action="store_true",
        help="Should the script download new data or use the local data"
    )

    args = parser.parse_args()
    [start, end] = args.range

    if start < 1:
        raise Exception(
            "Invalid start_index, start_index must be greater than zero")
    if end < start:
        raise Exception(
            "Invalid end_index, end_index can't be less than start_index")

    print(f"[:::::::]  Indexing from {start} to {end}  [:::::::] ")

    start_time = time.time()

    if args.download:
        download_pages(start, end)

    all_episodes = get_all_episodes()

    index_episodes(all_episodes)
    end_time = time.time()

    print(f"[:::::::]  COMPLETED  [:::::::]")
    print(f"Episodes: {len(all_episodes)}")
    print(f"Took: {(end_time - start_time)}s")


if __name__ == "__main__":
    main()

"""
python index/main.py --range 3 3
"""
