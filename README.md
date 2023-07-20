
Plex proxy that merges movies and shows recommended rows on home.
Make sure you have collections/recommended rows with the same name in both movies and shows (aka trending) as it will be merged by name.

Run cargo with your plex url as APP_HOST env, ex:

```
APP_HOST=http://4452-4-25-217.01b0839de6734738.plex.direct:42405 cargo run
```

add your proxy url to plex "Custom server access URLs" (ex http://0.0.0.0:3001)

then access your proxy url http://0.0.0.0:3001

![plot](./example.png)