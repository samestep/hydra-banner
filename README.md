# ❄️✨ Hydra Banner 

Motivation: I want a way to quickly glance at Hydra build status directly from PRs and Issues (since I found that a lot of people usually paste Hydra links, and that's another step I need to take before getting to the Hydra build report). So, here it is, directly in the PR/Issue!

I created this project to improve my own (and hopefully others) workflows and PR/Issue quality in Nixpkgs. Any usage or adoption of the project would make me happy!

### Disclaimer

The hosted instance ([hydra-banner.harinn.dev](https://hydra-banner.harinn.dev)) **aggressively caches** generated SVGs to reduce load on Hydra and keep it focused on build jobs (first-time generation always reaches Hydra). Note that this may cause banners for unfinished builds to update with a delay.

- 1 month for **finished** builds (HTTP 200)
- 4 hours for **unfinished** builds (HTTP 202)
- 2 hours for Hydra 50x errors (HTTP 502)

Hydra maintainers — if this service is overloading Hydra (User-Agent: `hydra-banner/<version> (+https://github.com/MiniHarinn/hydra-banner)`) please reach out to me at `matrix:@harinn:matrix.org`

## Usage

### API Endpoint:

```text
https://hydra-banner.harinn.dev/build/<build-id>
```

Embed in a PR or issue:

```text
# Not clickable
![](https://hydra-banner.harinn.dev/build/<build-id>) 

# Click to nav to Hydra
[![](https://hydra-banner.harinn.dev/build/<build-id>)](https://hydra.nixos.org/build/<build-id>) 
```

### Generate markdown for Hydra build(s):

```bash
nix run github:MiniHarinn/hydra-banner#getmd <build-id|url> [build-id|url]...
```

Example:

```bash
nix run github:MiniHarinn/hydra-banner#getmd <build-id-1> <build-id-2> <build-id-3>
```

Output (ready to paste into a PR or issue):

```text
[![](https://hydra-banner.harinn.dev/build/<build-id-1>)](https://hydra.nixos.org/build/<build-id-1>)
[![](https://hydra-banner.harinn.dev/build/<build-id-2>)](https://hydra.nixos.org/build/<build-id-2>)
[![](https://hydra-banner.harinn.dev/build/<build-id-3>)](https://hydra.nixos.org/build/<build-id-3>)
```

### Grab latest build (glb):

Don't have a build ID? `glb` (grab-latest-build) looks up the latest Hydra build for a nixpkgs attribute and generates the markdown directly.

```bash
nix run github:MiniHarinn/hydra-banner#glb <attr> [attr]...
```

Example:

```bash
nix run github:MiniHarinn/hydra-banner#glb hello ripgrep
```

Output (ready to paste into a PR or issue):

```text
[![](https://hydra-banner.harinn.dev/build/<build-id-1>)](https://hydra.nixos.org/build/<build-id-1>)
[![](https://hydra-banner.harinn.dev/build/<build-id-2>)](https://hydra.nixos.org/build/<build-id-2>)
```

## Example Output

[![](https://hydra-banner.harinn.dev/build/325973022)](https://hydra.nixos.org/build/325973022)
[![](https://hydra-banner.harinn.dev/build/326196302)](https://hydra.nixos.org/build/326196302)
[![](https://hydra-banner.harinn.dev/build/324307594)](https://hydra.nixos.org/build/324307594)

## Related Projects
- https://github.com/NixOS/hydra
- https://github.com/NixOS/nixpkgs

## Contributions

Issues and PRs are welcome. I just want to make the workflow around Hydra better for everyone in Nixpkgs!

---

<p align="center">Made with ❤️ by <a href="https://github.com/MiniHarinn">@MiniHarinn</a> and his passion in Nix</p>
