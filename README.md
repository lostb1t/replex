# Replex

![plot](./examplewithhero.png)

Plex proxy with the following features:

- Merge movies and shows from hubs on home.
- Remove watched items from hubs in home and library recommended
- Choose between hub styles, shelf (default) or hero.

Make sure you have collections/recommended rows with the same name in both movies and shows (aka trending) as it will be merged by name.

### Usage example

Run the docker image.

```
docker run --rm -it -p 80:80 -e REPLEX_HOST="http://10.0.0.3:42405" ghcr.io/sarendsen/replex-nginx:latest
```

add your proxy url to plex "Custom server access URLs" (ex http://0.0.0.0:80)

then access your proxy url http://0.0.0.0:80

NOTICE: this isnt a fully fledged proxy and doesnt aim to be. I suggest putting it behind a proper (reverse) proxy and only route the following paths (and it subpaths) to this app. 

- /hubs
- /replex

If you dont have a reverse proxy an docker image including nginx exists at ghcr.io/sarendsen/replex-nginx and a version without nginx at ghcr.io/sarendsen/replex.

### Settings
Settings are set via [environment variables](https://kinsta.com/knowledgebase/what-is-an-environment-variable/) 

| Setting        	       | Default 	| Description                                                            	|
|--------------------------|------------|---------------------------------------------------------------------------|
| REPLEX_HOST              | -      	| Plex target host to proxy                                             	|
| REPLEX_INCLUDE_WATCHED   | false    	| If set to true, remove watched items from hubs.                        	|

### Hub style

You can change the hub style to hero elements by setting the label "REPLEXHERO" on an collection. 

### notes

to force plex clients to use your proxy you either use a custom domain with ssl or use your server ip and set it in Custom server access URLs and disable remote access (you loose automatic ssl but you can provide it yourself with a reverse proxy and custom domain)