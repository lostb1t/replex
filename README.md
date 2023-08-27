# Replex

Remix your plex recommendations.

![plot](./examplewithhero.png)

## Features

- Merge recommendations on home into one from different libraries. Aka have movies and shows in a single row.
- Remove watched items from recommendations.
- Choose between styles, shelf (default) or hero.
- Auto load artwork for hero styles.
- Disable user state: remove unwatched markers from artwork.
- Disable leaf count: remove episode count from artwork.
- Hot cache: auto refreshed cache for home and library recommended.
- Force maximum quality.
- Auto select version based on resolution of the client.
- Works on every client/app not only plex web!
- Plays nice with PMM (and without).

!!This does not alter your plex data in anyway. it only alters outgoing api requests. All your collections or rows are kept intact!!


## Installation

Run the docker image with REPLEX_HOST set to your plex instance.

```
docker run --rm -it -p 3001:80 -e REPLEX_HOST="http://PLEXIP:PLEXPORT" ghcr.io/sarendsen/replex:latest
```

add your proxy url to plex "Custom server access URLs" (ex http://0.0.0.0:3001)

then access your proxy url http://0.0.0.0:3001

Docker compose example including plex:

```yml
version: "3"
services:
  plex:
    image: lscr.io/linuxserver/plex:latest
    container_name: plex
    environment:
      - PUID=1000
      - PGID=1000
      - TZ=Etc/UTC
      - VERSION=docker
      # claim from https://plex.tv/claim 
      - PLEX_CLAIM=
    ports:
      - 32400:32400
     volumes:
       - /path/to/library:/config
       - /path/to/tvseries:/tv
       - /path/to/movies:/movies
    restart: unless-stopped
  replex:
    image: ghcr.io/sarendsen/replex:latest
    container_name: replex
    environment:
      REPLEX_HOST: http://plex:32400
    ports:
      - 3001:80
    restart: unless-stopped
    depends_on:
      - plex
```

## Settings

Settings are set via [environment variables](https://kinsta.com/knowledgebase/what-is-an-environment-variable/) 

| Setting        	          | Default 	| Description                                                            	  |
|---------------------------|----------|---------------------------------------------------------------------------|
| REPLEX_HOST               |        	 | Plex target host to proxy.                                             	  |
| REPLEX_HERO_ROWS          |        	 | Comma seperated list of hubidentifiers to make hero stlye                  |
| REPLEX_INCLUDE_WATCHED    | false    | If set to false, hide watched items for recommended rows                                     |
| REPLEX_FORCE_MAXIMUM_QUALITY    | false    | This will force clients to use the maximum quality. Meaning that if a client requests anything other then the maximum quality this will be ignored and the maximum quality (direct play/stream when server allows for original) is used instead. This doesn't prevent transcoding. It only sets the bitrate to original quality. So if a client needs a different codec, container or audio it should still transcode.                                                                                                 |
| REPLEX_AUTO_SELECT_VERSION    | false    | If you have multiple versions of a media item then this setting will choose the one thats closest to the client resolution. So a 1080p TV will get the 1080P version while 4k gets the 4k version. A user can still override this by selecting a different version from the client.                                    |
| REPLEX_DISABLE_USER_STATE | false    | Remove unwatched markers from artwork.|
| REPLEX_DISABLE_LEAF_COUNT| false    | Remove episode count label from show artwork.                              |
| REPLEX_DISABLE_RELATED  | false | See: https://github.com/sarendsen/replex/issues/26.        |
| REPLEX_REDIRECT_STREAMS  | false    | Redirect streams to another endpoint.                                      |
| REPLEX_REDIRECT_STREAMS_URL  | REPLEX_HOST    | Alternative streams endpoint                                         |
| REPLEX_CACHE_TTL          | 1800    	 | Time to live for caches in seconds. Set to 0 to disable (not recommended).  |
| REPLEX_CACHE_ROWS         | true       | Cache rows            	                            |
| REPLEX_CACHE_ROWS_REFRESH | true     | Auto refresh cached rows           	                 |

## Mixed rows

Custom collections with the same name from different libraries will be merged into one on the home screen,
So an collection named "Trending" in the Movie library will be merged with an collection named "Trending" from a shows library on home.

Note, this does not work on builtin recommendations. As i personally dont see then need of mixing those. 
You can recreate the builtin rows with smart collections if you wish to have that functionality, or with PMM ofcourse.

## Row style

For custom collections you can change the hub style to hero elements by setting the label "REPLEXHERO" on an collection.

For built in rows you can use the hubidentifier in the `REPLEX_HERO_ROWS` env like so `REPLEX_HERO_ROWS="movies.recent,movie.recentlyadded"`
This also works for collections.

Note: hero style elements uses coverart from plex. Banner or background is not used.


## Remote access (force clients to use the proxy)

Because this app sits before Plex the builtin remote access (and auto SSL) will not work and needs to be disabled.

You have 2 options to provide remote access.

1. By ip http://[replexip]:[replexport]

   This option has 2 downsides. One it has no SSL so your connections will be insecure. Second app.plex.tv will not work. As browsers do not allow unsecure connections from a securew website. Gf you want to to use the web ui you can access it by ip. All other clients should work with unsecured connections. NOTE: If remote access was enabled before, then plex witll still try to use https for it. Even if disabled.

2. Custom domain (reverse proxy)

   You can setup a reverse proxy with a custom domain. This solves both the issues from option 1.
   This is the prefered way.

For both options set your domain or ip(with port) in the 'Custom server access URLs' field under network and make sure to disable remote access under remote access. 
Use https instead of http if you using a custom domain, otherwise app.plex.tv wont work.

## Reverse proxy

If you have a reverse proxy running and only want to route the necessary paths, you can do so. (and all subpaths unless otherwise stated).

- /hubs (excluding /hubs/search)
- /replex
- /video/:/transcode/universal/decision (if force maximum quality is enabled) 

Paths are subject to change. for an uptodate list see [routing](src/routes.rs)

## Redirect streams

If you have for example an appbox it might not be ideal to stream media through replex. As that will take a lot of network resources.
You can redirect streams by enabling `REPLEX_REDIRECT_STREAMS` and optionally set `REPLEX_REDIRECT_STREAMS_URL` if it needs to be different from REPLEX_HOST

## Known limitations/issues

- hero rows on Android devices dont load more content. so hero rows have a maximum of 100 items on Android.
- when include_watched is false a maximum item limit per library is opposed of 250 items. So if you have a mixed row of 2 libraries the max results of that row will be 500 items.
- disable_user_state: For movies this works in the webapp. Shows work accross clients