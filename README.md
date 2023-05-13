# Tunefire

⚠️ Disclaimer: This project is a work in progress and not useful yet.

Tunefire is a modular tag-based music streaming app that connects to multiple streaming services.

It aims to make it easy to manage your music by letting you tag songs with custom abstract attributes such as speed, cheerfulness, loudness, or any other feature you choose.  
By filtering your library with queries, you can create automated playlists to perfecly match your mood.

### Multi-source

Tunefire's streaming client is plugin-based, allowing it to connect to multiple streaming services.  
Here is the current implementation status of each planned streaming service.

| service     | Streaming | Search | Playlist import |
| ----------- | --------- | ------ | --------------- |
| Local files | ✅         | ❌      | ❌               |
| Soundcloud  | ✅         | ✅      | ✅               |
| Youtube     | ⚠️        | ❌      | ✅               |
| Spotify     | ❌         | ❌      | ❌               |


✅ Working  
⚠️ Working but might randomly break  
🚧 Work in progress  
❌ Unimplemented  

# HubDJ (Draft)

This repo also contains HubDJ, an app that lets you host listening sessions with you friends, letting you take turns playing songs.

# Installation

Compiled binaries for Linux, MacOS and Windows under the [releases section](https://github.com/Azorlogh/tunefire/releases/).

# Author's note

I am working on this project solely in my free time, therefore you can expect Tunefire's development progress to be quite sporadic.  
The primary goal is to create an app that I can use for my daily listening sessions, but it has yet to reach that point.  

### Acknowledgement
Thanks to Jan Pochyla for [his excellent spotify client Psst](https://github.com/jpochyla/psst) which served as a high-quality example of a real-world app made using druid.  
Thanks to all the library authors & contributors for making it possible for anybody to create apps of this scale in a reasonable time.
