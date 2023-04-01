import time
import argparse
import requests
import json
from bs4 import BeautifulSoup as BS
from dataclasses import dataclass
from typing import List, Dict, Tuple
import re

LINUX_UNPLUGGED_URL = "https://linuxunplugged.com"
EPISODES_FILENAME = "episodes_index.json"
BY_TAG_FILENAME = "tags_index.json"


@dataclass
class Episode:
    id: int
    title: str
    date: str
    duration: str  # metadata
    tags: List[str]
    url: str

    def to_json(self) -> str:
        return json.dumps(self.__dict__)


def filter_empty_items(str_list: List[str]) -> List[str]:
    return list(
        filter(
            lambda x: len(x) > 0,
            [s.replace("\n", "").strip() for s in str_list]
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
    return response.content


def parse_page_content_to_episodes(content: str) -> List[Episode]:
    soup = BS(content, "html.parser")
    episode_selector = "list-item prose"
    html_episode_list = soup.find_all("div", class_=episode_selector)
    episodes_out: List[Episode] = []

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
        times = filter_empty_items([d.text for d in span_elements[0]])
        date = parse_date(times[0])
        duration = times[1].strip()

        # get tags
        tags = filter_empty_items(span_elements[1].text.split(","))

        episodes_out.append(
            Episode(episode_id, title, date, duration, tags, episode_url))

    return episodes_out


def get_episodes_from_range(start: int, end: int) -> List[Episode]:
    assert start > 0
    assert end > start

    all_episodes: List[Episode] = []

    for page_id in range(start, end):
        print(f"Indexing page: {page_id}")
        content = get_page_content(page_id)
        episodes = parse_page_content_to_episodes(content)
        all_episodes.extend(episodes)

    return all_episodes


def index_episodes_from_range(start: int, end: int) -> Tuple[List[Episode], bool]:
    try:
        # the juicy stuff
        episodes = get_episodes_from_range(start, end + 1)
        save_episodes_to_json(episodes)

        tag_map = generate_tag_map(episodes)
        save_tag_map_to_json(tag_map)
    except Exception as e:
        print(f"Error: {e}")
        return ([], False)

    return (episodes, True)


def save_episodes_to_json(episodes: List[Episode]):
    episodes_json = ",\n".join(
        [f"\"{ep.id}\": " + ep.to_json() for ep in episodes])

    content = "{" + episodes_json + "}"
    with open(EPISODES_FILENAME, "w") as f:
        f.write(content)


def generate_tag_map(episodes: List[Episode]) -> Dict[str, List[Episode]]:
    tag_map: Dict[str, List[Episode]] = {}

    for ep in episodes:
        for tag in ep.tags:
            episode_list = tag_map.get(tag, [])
            episode_list.append(ep)
            tag_map[tag] = episode_list
    return tag_map


def save_tag_map_to_json(tag_map: Dict[str, List[Episode]]):
    entries: List[str] = []

    for tag, episodes in tag_map.items():
        episodes_json = ",\n".join([ep.to_json() for ep in episodes])
        episodes_json = f"[{episodes_json}]"

        assert len(tag) > 0

        entry = f"\"{tag}\": {episodes_json}"
        entries.append(entry)

    content = ",\n".join(entries)
    content = "{" + f"\n{content}\n" + "}"
    with open(BY_TAG_FILENAME, "w") as f:
        f.write(content)


def main():
    parser = argparse.ArgumentParser()

    # The script uses this to know the range of pages to request
    parser.add_argument(
        "--range",
        default=[1, 1],
        type=int,
        nargs=2,
        help="range of pages to parse, [s,e] where s and e are integers and e is inclusive"
    )

    args = parser.parse_args()
    [start_index, end_index] = args.range

    if start_index < 1:
        raise Exception(
            "Invalid start_index, start_index must be greater than zero")
    if end_index < start_index:
        raise Exception(
            "Invalid end_index, end_index can't be less than start_index")

    print(f"indexing from {start_index} to {end_index}")

    start_time = time.time()
    episodes, ok = index_episodes_from_range(start_index, end_index)
    end_time = time.time()

    if ok:
        print(f"-- completed --")
        print(f"processed episodes: {len(episodes)}")
        print(f"took: {(end_time - start_time)}s")


if __name__ == "__main__":
    main()

"""
python index/main.py --range 3 3
"""
