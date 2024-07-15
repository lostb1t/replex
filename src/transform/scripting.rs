pub struct ScriptingMediaContainer {}

impl MediaContainer {
    pub fn get_size(&mut self) -> i64 {
        // rhai::Dynamic::from(self.size.unwrap_or(()))
        // to_dynamic(self.size).unwrap()
        self.size.unwrap_or(0)
    }
    pub fn set_size(&mut self, value: i64) {
        self.size = Some(value);
    }
}

#[derive(Default, Debug)]
pub struct MediaContainerScriptingTransform;

#[async_trait]
impl Transform for MediaContainerScriptingTransform {
    async fn transform_mediacontainer(
        &self,
        item: MediaContainer,
        plex_client: PlexClient,
        options: PlexContext,
    ) -> MediaContainer {
        let config: Config = Config::figment().extract().unwrap();
        if config.test_script.is_none() {
            return item;
        }

        let mut media_container: Dynamic = to_dynamic(item).unwrap();
        let mut context: Dynamic = to_dynamic(options).unwrap();
        let mut engine = Engine::new();

        engine
            .register_type_with_name::<Dynamic>("MediaContainer")
            .register_type_with_name::<Dynamic>("PlexContext");

        let mut scope = Scope::new();
        scope.push("media_container", media_container);
        scope.push("context", context);

        engine
            .run_file_with_scope(&mut scope, config.test_script.unwrap().into())
            .unwrap();
        let result = from_dynamic::<MediaContainer>(
            &scope.get_value::<Dynamic>("media_container").unwrap(),
        )
        .unwrap();
        result
    }
}