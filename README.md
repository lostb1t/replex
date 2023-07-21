# Replex (WIP)

![plot](./example.png)

Plex proxy with the following features:

- Merge movies and shows from hubs on home.
- Remove watched items from hubs in home and all library home's

### Settings
Settings are set via [environment variables](https://kinsta.com/knowledgebase/what-is-an-environment-variable/) 

| Setting        	       | Default 	| Description                                                            	|
|--------------------------|------------|---------------------------------------------------------------------------|
| REPLEX_HOST              | -      	| Plex host we want to proxy                                             	|
| REPLEX_INCLUDE_WATCHED   | false    	| If set to true, remove watched items from hubs.                        	|


Make sure you have collections/recommended rows with the same name in both movies and shows (aka trending) as it will be merged by name.

### Usage

(Docker images are coming)

Run cargo with your plex adress as APP_HOST env, ex:

```
APP_HOST=http://10.0.0.5:32400 cargo run
```

add your proxy url to plex "Custom server access URLs" (ex http://0.0.0.0:3001)

then access your proxy url http://0.0.0.0:3001

fyi: this isnt a fully fledged proxy and doesnt aim to be. I suggest putting it behind a reverse proxy and only route the following paths (and it subpaths) to this app.

- /hubs
- /replex
