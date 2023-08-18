
// filter 4k media for smaller resolutions
fn filter_4k(media, context) {
    if media.videoResolution == "4k" {
        for (resolution, i) in context.screen_resolution {
            if resolution.height < media.height && resolution.width < media.width {
                return false;
                break;
            }
        } 
    }
    return true;
};

media_container.Metadata[0].Media = media_container.Metadata[0].Media.filter(|v| filter_4k(v, context));
