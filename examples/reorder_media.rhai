
//
// Reorder media based on resolution, most closest to the client resolution on top.
//

fn sort_media(x, y, context) {
    let device_density = context.screen_resolution[0].height * context.screen_resolution[0].width;
    let current_density = x.height * x.width;
    let next_density = y.height * y.width;
    let q = abs(current_density - device_density);
    let qq = abs(next_density - device_density);

    if q > qq {
        return 0
    } else {
        return -1
    }
}

fn reorder_media(media, context) {
    media.sort(|x, y| sort_media(x, y, context));
    return media;
};

if media_container.Metadata != () {
    for (item, index) in media_container.Metadata {
        if media_container.Metadata[index].Media != () {
            media_container.Metadata[index].Media = reorder_media(media_container.Metadata[index].Media, context);
        }
    }
}

if media_container.Hub != () {
    for (hub, hub_index) in media_container.Hub {
        for (item, index) in media_container.Hub[hub_index].Metadata {
            if media_container.Hub[hub_index].Metadata[index].Media != () {
                media_container.Hub[hub_index].Metadata[index].Media = reorder_media(media_container.Hub[hub_index].Metadata[index].Media, context);
            }
        }
    }
}