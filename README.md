
Plex proxy that merges movies and shows recommended rows on home.
Make sure you have collections/recommended rows with the same name in both movies and shows (aka trending) as it will be merged by name.

Run cargo (or build a release) with your plex adress as APP_HOST env, ex:

```
APP_HOST=http://10.0.0.5:32400 cargo run
```

add your proxy url to plex "Custom server access URLs" (ex http://0.0.0.0:3001)

then access your proxy url http://0.0.0.0:3001

![plot](./example.png)