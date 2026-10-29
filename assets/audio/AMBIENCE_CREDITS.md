# Ambient audio credits

Source: **The Shop** by **LEGIT Audio**, published November 1, 2020.

- Creator/source page: https://opengameart.org/content/the-shop
- Download: https://opengameart.org/sites/default/files/legit_audio_-_the_shop_free_sfx_wav.zip
- License: CC0 1.0 Universal, https://creativecommons.org/publicdomain/zero/1.0/
- License confirmed on the creator's OpenGameArt page on October 28, 2026. The CC0 grant applies to these free OpenGameArt samples, not the paid full library.

`office-ambience.wav` derives from `TheShopCollection_convenience_store_drinks_fridge_drone.wav`.
`breakroom-ambience.wav` derives from `TheShopCollection_convenience_store_drinks_fridge_drone_2.wav`.

Edits: mono mixdown, 16 kHz resampling, eight-second loop with a one-second seam crossfade, DC removal and peak normalization. The loop builder is `tools/speech/build_ambience.py`. These are subtle appliance/ventilation drones rather than recordings of identifiable dispatch calls. Playback is local; no asset streaming or network request occurs during gameplay.



## Recorded exterior audio (October 29 revision)

The previous generated noise bed and engine harmonics have been removed.

- `street-muffled.wav`: “Street Corner_P29_2” by **Eric Matyas**, https://soundimage.org/amb-urban-ambience/ . Source https://soundimage.org/wp-content/uploads/2019/09/Street-Corner_P29_2.mp3 . Soundimage International Public License: https://soundimage.org/sample-page/ (includes disclaimer of warranties). Credit also appears in the game's menu, as required by https://soundimage.org/attribution-info/ . Edited: first 48 seconds, mono 16 kHz, 100 Hz high-pass / 1100 Hz low-pass, 1.1× post-filter gain and one-second endpoint fades. Contains street voices, traffic and horns; no generated dialogue.
- `car-engine.wav`: “racing car engine sound loops”, `loop_0.wav`, by **domasx2**, https://opengameart.org/content/racing-car-engine-sound-loops . CC0 1.0: https://creativecommons.org/publicdomain/zero/1.0/ . Source https://opengameart.org/sites/default/files/loop_0.wav . Edited: mono 16 kHz, 45 Hz high-pass / 900 Hz low-pass and reduced gain. The creator states the samples were remade using a public-domain recording in the July 24, 2011 comments.
- Short intermittent horn and office telephone tones are original synthesized dual-frequency effects. They contain no noise bed. Recorded car sources follow visible vehicles; the exterior recording and horns get louder near windows. Voice capture mutes ambient layers and caller playback ducks them.
