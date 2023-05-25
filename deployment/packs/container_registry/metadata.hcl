app {
  url    = "https://github.com/davidkern/hab"
  author = "The Hab Maintainers"
}

pack {
  name        = "container_registry"
  description = "Runs a docker registry on host port 5000"
  url         = "https://github.com/davidkern/hab/deployment/packs/container_registry"
  version     = "0.0.1"
}
