# Ambient audio credits

Source: **The Shop** by **LEGIT Audio**, published November 1, 2020.

- Creator/source page: https://opengameart.org/content/the-shop
- Download: https://opengameart.org/sites/default/files/legit_audio_-_the_shop_free_sfx_wav.zip
- License: CC0 1.0 Universal, https://creativecommons.org/publicdomain/zero/1.0/
- License confirmed on the creator's OpenGameArt page on October 28, 2026. The CC0 grant applies to these free OpenGameArt samples, not the paid full library.

`office-ambience.wav` derives from `TheShopCollection_convenience_store_drinks_fridge_drone.wav`.
`breakroom-ambience.wav` derives from `TheShopCollection_convenience_store_drinks_fridge_drone_2.wav`.

Edits: mono mixdown, 16 kHz resampling, eight-second loop with a one-second seam crossfade, DC removal and peak normalization. The loop builder is `tools/speech/build_ambience.py`. These are subtle appliance/ventilation drones rather than recordings of identifiable dispatch calls. Playback is local; no asset streaming or network request occurs during gameplay.

City ambience and passing-car engines are original procedural audio generated locally by `src/ambience.rs`: filtered noise, low engine harmonics, a distant horn and smooth loop boundaries. No third-party recording, voice sample, AI speech service or network request is used for these layers. Car sources move with the visible vehicles; city/room ambience follows the existing volume, mute and speech-ducking settings.
