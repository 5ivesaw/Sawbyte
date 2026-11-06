/**
 * sawbyte - Final Minecraft 26.3 Interactive Engine
 * ============================================================================
 * 1. Full-Screen 2.5D Shader-Lit Minecraft World Background (#bg-canvas):
 *    - Volumetric golden-hour sunbeams / shader lighting, drifting blocky clouds,
 *      swaying oak/cherry voxel trees, shimmering shader water river, grass hills,
 *      and underground cliffside ore veins (Diamond, Emerald, Gold, Redstone, Iron)
 *    - Animated Steve, Alex, and Technoblade (with real avatars from mc-heads.net /
 *      minotar.net + articulated 2.5D bodies, swinging arms, walking legs, crown,
 *      and live activity bubbles) walking, mining, fishing, and farming potatoes!
 *    - Every block & tree in the 2.5D background is breakable on click/tap with
 *      official destroy_stage_0..9.png cracks, debris particles, floating drops,
 *      and 2.0-second respawn!
 * 2. Slower, Natural Login Intro + Singleplayer Misclick + Technoblade Vortex:
 *    - Accidentally clicks Singleplayer -> opens Select World -> clicks Cancel ->
 *      clicks Multiplayer -> Add Server -> pastes https://5ivesaw.github.io/Sawbyte
 *      & names Sawbyte -> Done -> selects Sawbyte -> Join Server
 *    - During Loading terrain..., dozens of Minecraft blocks & items swirl in a
 *      3D vortex and assemble into Technoblade!
 * 3. 15-Second Rotating Weapon/Tool Cursor (Desktop + Mobile Touch-to-Swing):
 *    - Rotates every 15s, follows 1.9+ attack cooldowns, plays item-specific sounds
 * 4. 100% True Non-Hardcoded Stochastic disc_maker Studio:
 *    - Uses Bjorklund's Euclidean rhythm generator E(k, 16, rot), Markov melodic
 *      step-walk with Consonance Gravity, and random instrument voice allocation
 */

const SAWBYTE_REPO = '5ivesaw/Sawbyte';
const TARGET_MC_VERSION = '26.3';

const MC_ASSET_BRANCH_CANDIDATES = [
  '1.20.1',
  '1.21',
  '1.19.3'
];

let activeMcAssetBranch = '1.20.1';

const SKIN_DATA_URIS = {
  techno: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAE4AAACWCAMAAABHGU5tAAADAFBMVEWhnJzk2tXo3qzs3NHk3tqwlV7Xop6aZiJqXmHb2mDl3Kfh3d2jnJ/Xp2bToJ0uJyGpnyXgr7Ho3SdlZmZyJpafXVydmJfjV1nQmZthIyRVTCVtbW2UKlmsTU7z9GovLy9xoRLVvsDd12MWFhZzCgpTZdaTcYOgnJzd2mTo56RTYJ2eZmai2B3s9AAgFsc2T9GmISq5si3tJCheWFi4FRWBM424cSq/9/ffzp44ODggGrd6FRNgNFFKLshmmcyaJyWaVl2+tWS9usCq/6r/AADHQj7/f//kpH/d2hjXzSc+PUAAAP8/fz8/f784T712EBlufrpgc8B/v395gHxyhL93jMR///+RPTu5N0GrbZeurg+7t8TBPETHb3jKf4PUd4DcfoHClTjcvcAAAACsAQGWAQEBAQFFRUU3ODft7O39/v7XlpSnZ2d4AQHHuATT1dUoKCjw8/KSVlXvFRTKysrQiIripqXjeHjhmZiQhwS9swJWVlaYmJjTdnXv5gSlWVmlpab5/PfPaWpoZ2e0d3V2dnYZGhrZpqXltbbOwwS0qQWRSEm0tbTUt7X7/PabpaXvCQt/f3/khYfq2Az5/ffJe4WKiIiRlQdVVVV8SE2tFRermZekY1vHg3maZWStODbTpVPt5y6sSUmmXGTx6dNLOTatJyi1h4eqqqrQY1vs2Nb6/fSb2BjNWljapJfRmHHCrAwLBtM7Q0KNFBXIqTLkopvu6OmWY1bPmFTqxsbu5+w8PDxNKSpIZueYopu/f3++vr6xyQ7amqfjnKOMegy4e4GknaHV4uP/f3/jfYH2+dkCAOWVKCmRe3rs6E/u6m7o5o86QT5yFhJzKJJ7M3yndVCrmQ7PyC/pgnzw6a7q5eNDXu9Vc8pnZ2eRODaQeiqLiYuohjK8w8PQti/Jycna1NL89Av//9L5+dZGRkZZWVlmJSZqaWh4eHicnaCpGyK1QTqqVVWtpieipUOppKW5w7zBAQLFRUjMxkrNyMfoMzfjamz/qqr++0b//7b5/PLmKcTlAAABAHRSTlNhZO2d4/+v//Dopx6e/tvs9hr9oP/i2/9a//5c/6EKmP//bScD//UVoGL9W/8C///mBf8qA/8DBgVR/6L//wWrFBH/AwH/Ahpon/8BBAT/brezBP+1twJ0tIO6Of97Omvc+7MA/v7+/v7/Bv7+/v7+/v7+//7+/v/+/v79/v79/v4u/v79/v79/v79/v/9T/7/Av/+bv79/wP//v7+/v38/v7//v7//vsD//+O///+/v7//v7//zL//v8TBP///gQE//z//fn//wL/Kv/8/vz8+P/+/////v3//FH//wX9/q7//v4NLf4Hac7U/7HS//b/A///0/7//f5M//8D+QO081NoWAAAE7pJREFUeNqdmgdgE0fWx+VucMGY0EILJCGQXPpdcvdd73df772PZcmjtSQbyZItCWlVLFuWZGzLgrgXXIIDOBBMDSEQamghCSSkQEgIpJdLzyW592Z21SW++x6WVquVfvt/Zd7MrlAQZpSKwaD4czF4FXb+kYjBLPFX8C6hE6FQiASvHpuYyMk5FrxCc5aFRJHSIFkMH5xaTOlV8jWlV4hkChm3SwwFKTkmBneJuWJwQjyW/ZsV2dnTwFaOF9Hp5eWDXy0j94mntZ9li1ezV0yncM4gnvEYISHx6ytXaCyO0OCyZSsXB7/6anX5YE7wvqLp5a+/vhrt9elwOFt6ETztazgt5pzzTTtG15+eNj04EQyBW0AQQwCRcfAydLJhBVk52LH69dVXQ2Jo8WpuHYPTZ36ek11e/mp5aX5+aWnZI9lZgDv3Dcn2nf1sgt792bTTNd98I4pIi+LupmW+abtWlvmaO5qnTc/PzR8YGNixY8fmzZs1+zUaI1gnPGw2o82o0eS9MrD+6opm7bmV9Hafr3nwq6+yg0Eao47cTf+1YVpxns+XZ9zs0w4YjXl5eZ2dnUZ3p9vttrl7jZs374Az2Nw2RBt7x8oxFs9n/dTnW93R0XGSTsQ5ezud5pv2Lz0+37OC2+f7H2PvWZ+vxNb7CNirOwAi7GCu9xp70YDXgbub92s2D+Crk/Gx+/rrEOACwz5fgQDMV4aFD31nezt7m5u12uYdnb29nRxn5NpAbDnulmg0+zfhgVNirDp6+7Fgsafswx5fQ8Hws4CrtZf5zrqXu7XNANzRaevt3MxwmyFyEEyz2czUlZg15hJ8dQpKMTZ2f0b+5s7C+vq1Za8Iz/oaXukTwNke4VltMwAHEDfAcGbEoW3juNL9+xHXcYrGZZb8iP71XxauXXvDDWvrL2wvK7APn23w9QwjDp219dpKJGdBzzYz8DowAyWQsM/AgY4VdDGJxf0F/fs7C8/Wr72hfm39008vHO0vY1lBnHYAMmzbsbq8vPzcANrNP/zhHdeVQRSav/XQQ996HD7S8RtyexzuR+Qf7ry1HuStra9/5umFFVsL1tbPXi4ATNv86gCkN+/mF29+9NYG2G14+OEnn7zjQzzCcPCi47fx6n5OQoZbAQfOMnVdFYB75rpnmDotJuR3dz5686Mv4mvtw2B3+DAMQHvocTjasSwoxmYWBn9+/QVIBcN9Z2EFx10n4eCbv3shinvy4Sdl3EOI03YsExNwdDq4Cd7C89PXLezqyoQD+xM88riMa14R16CwJTAcs4LCH1TclQL36Asv4le1T4IxnPZxsAZtw2TzMhKf2e8Go7j6+icKChjuO5PNjNfMcC++gKnQal2FeWVlDQ0+rRZ2GxrwLzu4OK7ufkqy1z7xRH2cFSJOUqd9JC/vww8nJxvk/QY0+Vm7Pr5Bka/FrOy8svoEuzApfzf6zEg+X0PUtA3nppPvxjkLDUYMrpydoK/ep52MyIggGFAra4MDDeWJOJguiEhn22sLLjwRh5P80mrZS5+skmF8bFd7ruy3WeTX8TgE/m2tvaVluGf7hXoZ6YPvaH0NUSWcrp3UTk6++mrJtAvXXz+je0JMmMlY6cGjD3D24WGhr6CAE30NkwwwKeOwnw4MlJRs2rQpt6qqRlVZWTnrm6y7ryTiGFKw17bU2u1C7QMP1G4vgEwzb4BY8lnJQAnMG2AHDuzffwDMu6Gqqgpxb9/+UrI6jqutbbH3CIBrqQXb/uwr14M3I+vhiztLzbzTmbfBA9qUt0rCXaTr0uDcdjuIG5ZxswVhBnxeX1VTVbNTIxu04m11ZvMmeLdKDYc/EttoJhx3Fk3GVdXU7DQiR8OV4aNuA5hrpPBgDtmXRt1BO7NkXBXiJG3b6tCamgJer7fugLkpl85Mgxsbsvck4xrRraUaDXy7zuttks27rQ7DWJdFfp8OZ69NcHb+jD0jhfBVINXJpECAbVAjyPXOJEfS4DSsUGpjcG4NxMwIjtY1eetkXQEgBgKoFXB1MwnNgGtpicENG9vb2wHIcIyHoMDSpYFspq4OnJ1J0+H2o7oYXJ9gxOUOxwWaomEbg5ULiGM5Sa/OO5SgTkAY1oYGkgBPzMx1NvyUhKu7TGmauvO2SLgWwG0XwIxmjjPbYMC0aHjN1XXC5wTO3tSUkwnHRDF1Es7MccLLLfNrEYY4I5xUwpmbQlFAAi4AOBgXceoYzmx2o2z+0sucFbirUCsh8QhNh0N9PT2sATAcGwmwxBEYjqvTGO0CpgJrMdCudJ5n3TcR93uigJaCDQqGhtAncHUcgc6iOiMfa5ghhIHZKpTv5U4UheIWs2iPkdsE9JXjhFgcpAL0CJ3ug25jN6PlsTSDw7aKCuWnOTSLJONAHbaAnhichrtrNkKlHTwIT5I4o1Q1TYCrqHgvP1h0hdIE3LwY3DDDdfLla7sRl9yMp9F0a8bGJHUyrqKrSUyMXdsvZh4UhmAoCNuxhw4n48BXN1OLT8zVAxJO2WUMFYnxmZ0JK5UxASIHkw/PhNDZHoNjPmo2eXGAYI/ZBhNQnbOC8yrqQt9ErslgbXvkA/LtKTFY2tKzfViQTcJxkMZ8gGWTVdsmeN62ra7JqZRwSmeugrHa4Pkm9gquB3O2bu0v6B8d5cnoNEe6MO/tmk2bUJwGayTQBFi7pE6prFikGJ/5GnJEWANM/XrevPxFd/2SH65AXB/gOC0yOqC3s+EATdBmf3+0C2VxmFLpRXXBEM3Z+d//7PwlHIuxfjeGz2bWmCPy2BYrd+On/V1cUcQAOpqlEEM5+Ytazigrkkz5MujrM3IccxnaZ/ZOR3d3t3xarikC7KeKUuH4VmW1MhUOjtthVLRzTTt3blgP82OVw+pwdEVocVZKFbOrq6uV8JcSV1ExavTCpHCo2+qvQYMpzQDWxVOZwOv6L6L4K6UyDa6CvefNPazzt1qt/irJZJxSmeBsxafBkOI/tlajpfKWmdM99997rHtRHU7eTF12TOxiQmenoiLneEbcJ/g31A3qwFFmOoMh0NXVP5QcvHwYs+JxFrvqdOpYBTocHFdVtX49OrspoAvgeeJo782juxTkrmouLwNuiMeuZv2pk4OD2QadQadDXIK6T0NBUUGFa+OG/X6/AXFwCToIKPA3Bc4GHUpB/0mZEdffl5fnRz0RHDgLRG8SLp/1u9u2ZpTnRkfRPwgd4lZnw2uD39+diOvKEXftUlAptelx3Xv9KIipGzx5ChbYVVZr64lEXH9IDFFFaOLla6tDcVh1uZBbHBkw0BJxFYuwxSn2iS3XUmc1VPEKlqxG57C2uuNw0NvzcZUHDWq28po4NliZML6BMhxLwPUXMdyEWHpN3FLG4Y4yIIRSk4CbTW/CeXZKvE2ZyVsBYmdAEoJOnxo8WcW6SiyO93XyAZ8Y5x2/No7H7Xm8/ybhjDKOb7py7n8MccGi4FAmnNvv8Os4jZexjHtvyNkfxUGZsEXFxBSdnQk3pmNVUpOAw0o8LERxncEptuQRc+miTLkw4gCVquR5KOPTEg7PYo/idobI/egsDVE5tcr0OO4stCfe4Pm4zbVHm1MuKWLrO1CY/3/F1cgb7AGGgHcogts4USStPmeSn2RKbW8UJwPRWb8bJ2xuW/vt3pr7RRY7SsUUo1YZqYLeBHXrpUE2xD/xst3d7f9YlxUsEqXV5y30+8rE/q6MbNwRHITuFC9jHXQUXJksPBGG7mC4ZMgl+3bJS+2byI8jOGUSrjfXYJCH6vNSoSAO1Ql7rR9f8gfgfNK9BMTdR4o5rlqZAmfT/SxSxtkJuD7roaUBA8xLxaQtciHwGJleLeGUCTjYCo7IlF0jqzNIuHCrf+nHfmvY8Tm5JYJbRxTzk3BKeWNvPSRN2VDGg4Onn2fq9joQ58DB4bdaP6IxlylHSJHcQZXRpiMTOU7K7Hq+MXQ7mDrvJcPH/u6xsY9YN5FwsFCsrU4sZInZxefYSCeuyWXd2Nrd2g8f0vg1nTab05ZLY3DkJvr96vjKY9vRIbdmp3HsEiQu2tdZ+wz4f6ZDdU5u5ujNAMT9hCyKdlAmanSo1j3m0B0+fFjHxnqNRKrZkF1nmnuPM9dwGHCfbGQ0m2Yq9j4KJaH8MxFvR+1uTUAH2XdAn2MTPqpDURuazHPvWbVqzZo1TjjD+5I6m609nzxG4nCXC10n+voK5/cBCSXpdA5c5Ug4nW5DU/uXX9wDoDWrwNaY4nDGHBp344OSIqvHurfV6nFU6XJ1zGR1rBSMX8ikVRFcv4xzai7H3wugbdThslodYY9VJ5vD0eqAhQ5otFr3ClEUw9k4rovj6sSEWwsv0VkeKHSrJxzBMR9xVX0IDgir4nFc3ScVD9qwTPKj94w47haiCIdBhsdjYIst2fyOQyAy3JpK3WjX+xtZ6DRZ0Uxw3GskB4ZK2BMOO8A9hyGKe7d1r/WQw5aEC/ghaUs7jx4ozc8hUV85ro1chhiFXWHwzhqDu9Tq6B4LGHTtCTinw/ourtFyZ8K1F15/Jd2WaX232+pZDkM7Rl1AY3M7nQGdTpOAE1oPzflYZ6gK4jWrmISDfQOs9T0ecMzR2npJZ9gZ2N/ebnM++OCDG71JuFXO1netsH430PEiGnc9LKk7QovHwuETHuu7DrjkgstDyNiXJhPDaQAXLw7UOQKOOVY/q5DoLZ4Ijt2iqO3rO3FCo+k1mb788ksbM8QZ/a3uGNSae+aaAjCEHXuthpiCi8PtI/Pc9toevN40mY4eNWEBmEwm9HajzcpxOC6+mGtq2sCvVrrdLYJIfpUSR0nWWAveDxNs7YCD2gQzgccbnQ86u+e4Oeo5CVW1wSss3IqXr9G7lAk46ma31wT3UVP7URNIYzgbw1ndX8w1R1AaZ38Xn4K7/pP8IiUObA5ztk8wmdpNiDOx57lOTO0GadGJKHlNh80/pjMl4GhpFPecKWIMx1goTcNnTqV09Qkrzm+nxt1EbuthtxLsnab2dkkdbFjv5teKVYHuhcqINgb8cTocu12E9znstvYYHNuaAdWksR+PvYjm1E9FMp4SR8l5YwviahlONpsJHG/X2Ecjc6UysiCu2Hqm9t/S4Y7Q2ezOTq0zBmdzMmRkvV8dmX2VW98aUW1R3UvWpU7FB2Q5u6lTK0DhSTTotLbnnnvOdFcsDoCAGrleBTRVcTrcOloqCNshdsO2aNSA6NzYPxpZY0Lwtr711sgWJDGbQ9PhSKkdf/cYtksgiFocCmy+fg+qkkwNj1np1I2TeV430moZ7qht4/vHqxPuuuxRq7eorldtYdosFngqpG1tqUcFpcGsefmzjZALm014YI+0ooqxar1ejZJ+oFI95Qq3OiwqtScr+stHihu9hBb9b3afyuXSq+cn4s7sUTOcxTKiUoVhTnHB6wUkjTqQN74L/89FbqHKdcKiV8fizswfGRmxuFR6VRimY7VK7YI5xQPsBWR3enXQWPeRi+DP36n0M/ZItDNn9oyMbFGr1ZblFr0FFxuQBtx6VBbXnHQdRbI/J2/DyS0qiNN8CNdbe0ae2vLUUwizbPFADKwSLuxxWTwezzVwsL54EyKkxjDp9yALdalVrrDVo/Z41HrAOSwWlwfrBHDvpI2dVC9TLr0Kvq+3gCQVPiy4C2sEjDzk0uoBmMeDtQJba1b8dJGAW0duseixqtQul1o9orZ4wlbAuWCJYLGo9VhtsAc4C6hzgb+KeHlJOAWkz6Li/qj/dDksDlAhUFT8gYEDnAvygDvfi09tEm4OJAE/ZwEBahUuNSwulwoDCPlhRzBoLvQY3FddpCQj7m2MPP65MNrwBDQXVi/qYvK4LKYQgnoN3CyWVDXGCWKDSXVZLIiE0EH4VCiLnYjnw5KpUMZJ0KJnZQIRdKGTKEet8rjCiMZsuDwuD74HWxdu4kdtPK6NKNSNahYocNKCsjCVMLysDhUKhgLy8KBZeBgtCvlyLAVuN7lR38icRRSWAvMX9BV/pOY4lVQlkkETeCnDIJtVqWfyVFwVE4dis25US6PFJemKNOQ2mh73ZmUjyMNOhP8YC19ZphQYVOx46liYSh2mJANuRiXjcSlqTgPzkCmoOkaLx6ks6ct4HzmvlnAyR7I5lMyI8OOtKHaYJeAUevyxWK9Xx5lepSD0ndQ4VzF5LQ1uN/leY2UiT6/XN6qyCPlInwIHu7PoS2l/Y3y7kuGYuyrOQhzWviJJHT+8JLYJxP+wRWdVcp4+ggJYY+U7eMdFJWmOZYG9SRMvU6KWpZZwQJFQuIsTVtBSie8wXvRcer3ltZhcJBZKcaXsrswCU5+HcNMZjWhI1MdYY+ONGToKncEJehmF9gb4s44U6htlYISFR4vTq4NuXJls77AjFxsb43jyCd+OGbWJMxkNepJojQq2ULqRBUGixYhfQnenjR3Ia0zEqT8Hd9aRexvltDfGfcQipvshH6+8yZJEcYUibYPOKqoqU5r6cgYcbXtNH/NZvWWWYpd0JTMjNS42tUk4sluuZTivpVghyrfmdifJlm1BJlwbocwr9RvFt4j8DWnIXEyDeyeai2RnyXm6AIbOLH5Zv64tepriNLglmdSxe92fS0Udd+DexjS5iKY2BU5ybzzpLKIlDe58ZhxpG08lmr6RBqeIXA+kxqW0G1OnVj1LcXmK0D8ah3N6EuvN4qlMi9n0RskucUFh7PjSv3Hxc1B1vm2c/vE4AEJEdy14Qx0ZLkhZN77v/6WONwiAZi1RV8IFCivx8YQ7H38AuhoMgv26vCQAAAAASUVORK5CYII=",
  techno_seated: "data:image/png;base64,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",
  steve: "data:image/png;base64,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",
  alex: "data:image/png;base64,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"
};

let BAKED_BLOCK_SPRITES = {};
let worldIntroActive = false;
let sfxEnabled = true;
let audioCtx = null;

const SOUND_SHA1_CACHE = {
  'minecraft/sounds/random/orb.ogg': 'e9833a1512b57bcf88ac4fdcc8df4e5a7e9d701d'
};

const DECODED_BUFFER_CACHE = {};
const WORLD_TEX_IMAGES = {};

const ALL_MINECRAFT_DISCS = [
  'item/music_disc_pigstep.png',
  'item/music_disc_otherside.png',
  'item/music_disc_relic.png',
  'item/music_disc_creator.png',
  'item/music_disc_creator_music_box.png',
  'item/music_disc_precipice.png',
  'item/music_disc_5.png',
  'item/music_disc_13.png',
  'item/music_disc_cat.png',
  'item/music_disc_blocks.png',
  'item/music_disc_chirp.png',
  'item/music_disc_far.png',
  'item/music_disc_mall.png',
  'item/music_disc_mellohi.png',
  'item/music_disc_stal.png',
  'item/music_disc_strad.png',
  'item/music_disc_ward.png',
  'item/music_disc_11.png',
  'item/music_disc_wait.png'
];

const CURSOR_ARSENAL = [
  { id: 'netherite_sword', name: 'Netherite Sword', tex: 'item/netherite_sword.png', cooldownMs: 625, baseDamage: 8, affinity: 'sword', sound: 'entity/player/attack/sweep1.ogg', fallbackFreq: 340 },
  { id: 'diamond_sword', name: 'Diamond Sword', tex: 'item/diamond_sword.png', cooldownMs: 625, baseDamage: 7, affinity: 'sword', sound: 'entity/player/attack/sweep2.ogg', fallbackFreq: 380 },
  { id: 'golden_sword', name: 'Golden Pigman Sword', tex: 'item/golden_sword.png', cooldownMs: 625, baseDamage: 6, affinity: 'sword', sound: 'entity/player/attack/sweep3.ogg', fallbackFreq: 420 },
  { id: 'netherite_pickaxe', name: 'Netherite Pickaxe', tex: 'item/netherite_pickaxe.png', cooldownMs: 833, baseDamage: 11, affinity: 'pickaxe', sound: 'dig/stone1.ogg', fallbackFreq: 260 },
  { id: 'diamond_pickaxe', name: 'Diamond Pickaxe', tex: 'item/diamond_pickaxe.png', cooldownMs: 833, baseDamage: 9, affinity: 'pickaxe', sound: 'dig/stone2.ogg', fallbackFreq: 280 },
  { id: 'iron_pickaxe', name: 'Iron Pickaxe', tex: 'item/iron_pickaxe.png', cooldownMs: 833, baseDamage: 7, affinity: 'pickaxe', sound: 'dig/stone3.ogg', fallbackFreq: 250 },
  { id: 'netherite_axe', name: 'Netherite Axe', tex: 'item/netherite_axe.png', cooldownMs: 1000, baseDamage: 12, affinity: 'axe', sound: 'entity/player/attack/crit1.ogg', fallbackFreq: 190 },
  { id: 'diamond_axe', name: 'Diamond Axe', tex: 'item/diamond_axe.png', cooldownMs: 1000, baseDamage: 10, affinity: 'axe', sound: 'entity/player/attack/strong1.ogg', fallbackFreq: 210 },
  { id: 'mace', name: 'Heavy Mace', tex: 'item/mace.png', fallbackTex: 'item/golden_axe.png', cooldownMs: 1600, baseDamage: 22, affinity: 'all', sound: 'random/anvil_land.ogg', fallbackFreq: 120 },
  { id: 'trident', name: 'Trident', tex: 'item/trident.png', cooldownMs: 909, baseDamage: 9, affinity: 'all', sound: 'entity/player/attack/crit2.ogg', fallbackFreq: 460 },
  { id: 'diamond_shovel', name: 'Diamond Shovel', tex: 'item/diamond_shovel.png', cooldownMs: 1000, baseDamage: 8, affinity: 'shovel', sound: 'dig/grass1.ogg', fallbackFreq: 230 },
  { id: 'diamond_hoe', name: 'Diamond Hoe', tex: 'item/diamond_hoe.png', cooldownMs: 250, baseDamage: 5, affinity: 'hoe', sound: 'dig/cloth1.ogg', fallbackFreq: 520 }
];

let currentTool = CURSOR_ARSENAL[3];
let lastAttackTs = 0;
let lastChargeRatio = 1.0;
let toolCountdownSecs = 15;
let totalMinedBlocks = 0;

const JUKEBOX_DISCS = [
  { id: 'pigstep', title: 'Lena Raine: Pigstep', shortName: 'Pigstep', soundPath: 'records/pigstep.ogg', altSoundPath: 'music/game/nether/pigstep.ogg', tex: 'item/music_disc_pigstep.png' },
  { id: 'otherside', title: 'Lena Raine: otherside', shortName: 'otherside', soundPath: 'records/otherside.ogg', tex: 'item/music_disc_otherside.png' },
  { id: 'relic', title: 'Aaron Cherof: Relic', shortName: 'Relic', soundPath: 'records/relic.ogg', tex: 'item/music_disc_relic.png' },
  { id: 'cat', title: 'C418: cat', shortName: 'cat', soundPath: 'records/cat.ogg', tex: 'item/music_disc_cat.png' },
  { id: 'stal', title: 'C418: stal', shortName: 'stal', soundPath: 'records/stal.ogg', tex: 'item/music_disc_stal.png' }
];

let currentDiscIndex = 0;
let bgMusicAudio = null;
let bgMusicPlaying = false;
let bgMusicVolume = 0.18;
let userPausedMusic = false;

document.addEventListener('DOMContentLoaded', () => {
  initOfficialMinecraftFont();
  initMcAssetRegistry();
  initOptionalScreenshots();
  initMobileNavDrawer();
  initSoundEngine();
  initPigstepJukebox();
  initMinecraftIntroSequence();
  initWeaponToolCursor();
  initShader25DWorldCanvas();
  initDiscMakerStudio();
  initFloatingTooltips();
  initScrollReveal();
  initReleaseDownloader();
  initTerminalConsole();
  initBenchmarkTabs();
  initRespectsAndTechnoSoundboard();
});

// ==============================================================================
// 1. Official Minecraft Font, Mobile Menu & Multi-CDN Asset Resolver
// ==============================================================================

function initOfficialMinecraftFont() {
  if (!('FontFace' in window)) return;
  const fontSources = [
    'url(https://cdn.jsdelivr.net/gh/IdreesInc/Minecraft-Font@main/Minecraft.otf)',
    'url(https://raw.githubusercontent.com/IdreesInc/Minecraft-Font/main/Minecraft.otf)',
    'url(https://cdn.jsdelivr.net/gh/Magisk-Modules-Alt-Repo/MinecraftFont@main/Minecraft.ttf)'
  ];
  const font = new FontFace('MinecraftOfficial', fontSources.join(', '));
  font.load().then((loadedFace) => {
    document.fonts.add(loadedFace);
  }).catch(() => {});
}

function initMobileNavDrawer() {
  const toggleBtn = document.getElementById('mobile-menu-toggle-btn');
  const drawer = document.getElementById('nav-links-drawer');
  const label = document.getElementById('mobile-menu-toggle-label');
  if (!toggleBtn || !drawer) return;

  toggleBtn.addEventListener('click', () => {
    const isOpen = drawer.classList.toggle('mobile-open');
    if (label) label.textContent = isOpen ? 'Close' : 'Menu';
    playMcClick(1.1);
  });

  drawer.querySelectorAll('a.nav-link').forEach((a) => {
    a.addEventListener('click', () => {
      drawer.classList.remove('mobile-open');
      if (label) label.textContent = 'Menu';
    });
  });
}

function getTextureUrls(texPath) {
  const clean = texPath.replace(/^\/+/, '');
  const urls = [];
  for (const br of [activeMcAssetBranch, ...MC_ASSET_BRANCH_CANDIDATES]) {
    const rawUrl = `https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/${br}/assets/minecraft/textures/${clean}`;
    const cdnUrl = `https://cdn.jsdelivr.net/gh/InventivetalentDev/minecraft-assets@${br}/assets/minecraft/textures/${clean}`;
    const cloudUrl = `https://assets.mcasset.cloud/${br}/assets/minecraft/textures/${clean}`;
    if (!urls.includes(rawUrl)) urls.push(rawUrl);
    if (!urls.includes(cdnUrl)) urls.push(cdnUrl);
    if (!urls.includes(cloudUrl)) urls.push(cloudUrl);
  }
  return urls;
}

function attachMultiSourceImage(imgEl, urls) {
  let idx = 0;
  imgEl.crossOrigin = 'anonymous';
  const tryNext = () => {
    if (idx < urls.length) {
      imgEl.src = urls[idx++];
    }
  };
  imgEl.onerror = tryNext;
  tryNext();
}

function createTintedTextureCanvas(img, tintHex) {
  const w = img.naturalWidth || 16;
  const h = img.naturalHeight || 16;
  const c = document.createElement('canvas');
  c.width = w;
  c.height = h;
  const cctx = c.getContext('2d');
  cctx.imageSmoothingEnabled = false;
  cctx.drawImage(img, 0, 0, w, h);
  cctx.globalCompositeOperation = 'multiply';
  cctx.fillStyle = tintHex;
  cctx.fillRect(0, 0, w, h);
  cctx.globalCompositeOperation = 'destination-in';
  cctx.drawImage(img, 0, 0, w, h);
  cctx.globalCompositeOperation = 'source-over';
  return c;
}

function loadCachedTexture(key, urls, tintHex = null) {
  const img = new Image();
  let idx = 0;
  img.crossOrigin = 'anonymous';
  const tryNext = () => {
    if (idx < urls.length) {
      img.src = urls[idx++];
    }
  };
  img.onload = () => {
    if (tintHex && img.naturalWidth > 0) {
      WORLD_TEX_IMAGES[key] = createTintedTextureCanvas(img, tintHex);
    } else {
      WORLD_TEX_IMAGES[key] = img;
    }
    BAKED_BLOCK_SPRITES = {};
    if (typeof window.markWorldTerrainDirty === 'function') {
      window.markWorldTerrainDirty();
    }
  };
  img.onerror = tryNext;
  WORLD_TEX_IMAGES[key] = img;
  tryNext();
  return img;
}

function isTexReady(texOrCanvas) {
  if (!texOrCanvas) return false;
  if (texOrCanvas instanceof HTMLCanvasElement) return texOrCanvas.width > 0;
  return Boolean(texOrCanvas.complete && texOrCanvas.naturalWidth > 0);
}

const SAWBYTE_LOGO_DATA_URI = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAGAAAABgCAYAAADimHc4AAArt0lEQVR42u29eZQd93Xf+bm/Wt7S3ViJhQAp7gtAECQBEiRISdZm+UShIluK4jhyJNmW4+MkM/HYyfiPnMw5mTnJyR+ZTBIrieKxJ1FOlMSyLIuWJUqURNIWSRAAQZAgiY0kQBBLY20A3a/7LVX1u/PHr169qnr1XjckZzJZ+pwm0a9f1/vVb7n3e7/3e2/B//j6L/ol/5Vd97/0l/7/caJMeh1JB5j8N75hTe5e7Y+7KD/OAnjp/6sm3M8N8s/qNNjcgpd3pVT8u+petbSTZZHdnv99khtDeR50xO/+syyAST/Q3USjsSHE25nAw6JsAjYCE9n7BEELn6O5T5bsNrUwKZK+aCG9AlgURfBQEVAtzJKkr2nu3gbXFfd7rPuzbC1kaCYUBZH0s81gzBKL6hU1chKVN1ST55JufQ9cmvtxFkJ+hF2fAHi1iQ+LMb8iykcRWT7aVC71IFzbYdHSRpbCp2rlLcqIK2l2hdGfIqUrqCqovivCEyT6O1E0f6A8R3+WC5DtD78x9YjAPwT5YG6QkQ7GKWh/V2Y7mXT3Vn1maeZFB9NCuq2ldEiyv5LC/GnJHkuFhZbST+m4BBFFVTQ7celEp4NwI9PcIguILyKoag/0Kz7x3+90OieuZRFkiSbHAuLVpv6BEf4Ogq9KDKgIBjA6vCkHF09vqrST0tuqOi6CuhtM368okn3Ej41Fhg+lZGulmYnMPrq/ANm9FEeRAB6CQbkoJL8RdRb+3ZCpXsSRjvu9ZWJibeDV/1CMfE4HSMcDTPnsSnkqB/hIEPcjBeM87EVHbg79UQ3W0L6vOglIaTOIlLyFVg2zP4caAVMi5mckCJdp3PtuBWC4pgVwO7/RuMG33vdE5BFV7aavm8Lg3cSWna0bs+TcrhQdreRmUgYnJb+muQNQvrZm/00XWq7Zq+QBwtDoKR8PkeoF7YOG1FJIbMS813jBrTbpfWOxRfDGIp1ly1b5iXxfRLa4yZeg5NVEyju9iEEkfY+k5qr/FhkBIKWw0XT0/Olgk+Yv2l/T/gJLf2gF5MWiniJvhGTEOsownJI0TtCeGLPd+P5GG0dPpLDcLnUB+sGG+hI8ISI7VclPvuR2fd4rkRtMAXoUjraQX4b8zcnY6Rg6++5CZZMluU17LRis8i3yo0J18UB7YHYYE3Zt0vvTHExd1Al7QOLXJ/+eiPnfnYcnQJxX1CIyU5EMthQxvo4cvuaMUx4JjgWnOlhOTQ2PqDsDqprZuEJ8UF5PGQlQR/5acxHGuP1Q5coUsOmA3xu353ZXoSOpMj1hOLnJGtmfhd2S2jEdtRFTU199p0KVQVHK/ngJ5Et/ihURI2JEbZxg/HR4iaboKefhy/M6dBx0RAxQCiAWR15V77Ai+Nbq/qTb2pGjLnSUCXKTGYS/KyKbgRjwpICM+wZotNmQMtYYjUGWPPH5qxrfI+nFJN0Ofj0kmp/HRopXCxAjolqGYzLGDskA54+2DuVYTYc212hfGosxGz0/PGbj3itlf+CVIWfQbD6Amn+cvsmr2gMihcnth1hS2koyZnfItU2+M+rGN9jYkrTnmLxuOR/8zU/w0//XL3Ld7etl9sxlrp46j41ivCDEeKZoNvooQUZMcz7gGo12rjWOykyTwp0a9/7vcSbIAxKvPvElI96vpLbfL+MUF/iV4v8sfHKBY8UUj4xHtRzoDEFAwXiGpJeQ9OapL1/Gw7/0Id77tz7GuvdsoEuXkBoL7XkOfWsfu3/7Bxx9+nU06WHCBl7go1azQQ/C29JwVAtQaokQdqlUtQXxVZMPJ935p/O+oAQW1k34tfkjiGxMzY/J+IDqralL2BGFfagD7ys5n6FVBsAYQ9KNsHGbxsoVbP/M+3j0b/4UG++6mW7SodfuYoyHWsULPOq1JhbL288dZPfvfp/X/3AvnatXwGsQ1ENUFbW6CGe15C0u15AniBEJVPX/STpzX6haALf7axMfMMZ7RlXjygBCRtiH6t9X0cSqY2xxNvGeIW730KTN5NrrePDzH+CRX/4w62+/kR4RvVYHMcaFPn27p2CtIiLUmnUEw/TRd9n35WfZ/5XnuHxiGggIJhrOCyZ2Sf50kdMg41YxZxysgK/o0aTT2gJElKCgD8R+OPmbeOYfMTA/o6LapdjAyl2hOoq3SImVKIKkw7INa9n5hY/w4Bc+xJobryeyPXoLXTCCGDP28Km1qCrBRI2QGrMXZ3jla8+z93ee5tS+t90NNxqIMdglLkT1Vim+SaGAWHPsVd9vJsaypdebO9JnGgonwK9NfhljPotqlFsArQRmOhIPD41cS+dBcyPNQl8RFWtZcf0Ktv3CT/DIFz7KdRvX0U7axO0I8UxmCvMsUhWyzYILq1hrCWohtaBBt9vm8HdeZvdvf5+jT72KjXuYsIkX+mhii36i6va04COGXJdWBC65qDABCdTqx5Le3JOZ1clvcBOEvwrcliIgM8wZVtIDOiZcr8Deff4ix98YQeIEf/0kf/4rv8pPffYv0FjWZG5+liS2GN/L/qZME1ftzczMiTstapVetwsGNmy5hW2feT+3/+R9YGHm7bN0Z6+i6uHVghEbv4SgdMQx0IobziV6RMRT9BlNevv7qNMrBtDhr4oxN/bp5xHsyGIOR4ZMjoxHcmIEY8Gub/DG8jneOHQIz/NZt34NjUYday02sX3iftiMZTtThkmJvuk0jm2Luj3iOGb1rdez9ROPsvlTD1FbNsHMO+dYuHDJxRNhgJih8FcGHKBUcVkyPOuFf1pEPKP8iU16u/omKH8CMH7410TkhuwEFJkaGW8SR2z6fj5G8qx0+itjHNKJYpJoHmsEu3kFly/NcuCNIxw+eozYJqy5bhVTE5MoljhJUppNCthesly5KVlIKcaExr0/7kX0ul0m1i1n04e3sfWvPMqKm9YwN32Z2ZPnXDwRBhhjsuRQnv9bZBtWUZMqiGeV7+mYBfildAH0GgKPJZgfyY6PSXE9gsQLXbFRm6m1K9j28x/iJ//uX8ZbUWPmylWSKGJ2rsXhI2/z2sHDzM0vsHLVClYuX44RIU5sCiFkaJKLpJTkOL8c5yzGOeEoptftEk40uH3nvTzwufezfutNtGdazLx1jqTXwXg+JvD6EFbSOEJGMKuSy3RL9vGKFRFPlafyC1CIA7za5C5jzCMjYeiYVOUQ9NJhU4CARgk27oAE3LjjNrb/1fez5WceYeWGdVgSQDk1fZ49L+3ntVcP0braSsNgmJhssvWeTTz06DY2rl4DidIlxqIYTA6G2EJiS8rOuTTYPj62ifM39XqThIRjz77B7i99h0PffIloYR4TNDG+h1orY2dC8hmMLDyNBAmt1b9te3P/Zx95Di2AGPMIqpFTH4zc+boIaBvExvmvXkx9VZO7P/YAD3zuJ7j1fZupBU16SZdeu5eaFQjCAN/3uHBphn0vHeDl/a9zZeYKGAPtHsGfu4M7d2xmm1nP7fXraJqArsbEatPptqV0/VIgZsrhKWgWTzQQDGcOHOPFLz3Jgd97ns7lBUwtBNVqAFbBEqSp61iQAKu/Effm/kl/AYomKAh/2YjcgGQoqIpQW2JGNuedRNCox02fup/Pfe1/5SOf/wTLbllDr9uj1+1hFcTzUvsMSZLQjSImmk3uuvM27tm6iallU7Rb83SvdJC7VnJ2RcKBuVO8FV0iVstKf4Ipr46gxBl3VjZBVZaz5OhEBn6iGxH3ekxtXMX2xz/Als8+xuk3T3D58BlMEEh5j4lU5xHS160gHsr3RjthL/wl+j5AFzdBOiI4ziW3Mh8gEcxtX87b9jKtuausWrGSqallKEpibeG0IoKKkFiLxpbl9Uluu+VWlt19PSeu69LZ0CTwQ4z4zNouhztnea09zeWkzaTXZLU/QSCGWLUkOhn4AcmBmmGg527dqhKEAaEf8s7bx3nqyac5cfQEvDmLeiIjVrMIXCQzgUm6AN/NL4BfdS4L+oPR6GfwPimrIYR84CwC4huIlDOnznLm4Al27X2FB7ffxwP338t1K1bSjbr0oghE8DDUjY8CF+IFXpyb5sDCaabjOeS2SSRWksQiIgQSIATM2YjnWsfYu3CSu2pr2N58D7fXVhOKoWN7RFi83BzpmLSXtRZjDFMTU1y+fIXv/P532f39FyXqdtXM99z96WhLuwhhVni3X5l/lpy30jHXLCfctYSdPYMRBzNtNA/zCxhZTjg1wczlWb7z7R+wa88+Hti6he3b72PdmuugZ7mSdHm9fZY32tMc61xiQXsgHoF4SNehHyOCiIOICnjGJ1Afi3KgfZbX22e5MVjO9okbubu+jhVenUhjuppkWabybVkntqLRbBL1In74vR/yJ3/0DFenL+I1Gho0m1iJKLnbobz2EnmkigWQoipqBKlWQS9IXm7osLNCvNAFejRWrWDLJ97P1ON3su/Nw1w9dwlTr1GfnKA12+bZ7/2Q3btfZvsj98MDGzjYOctMsgAIgfGpSc1NtPQxf3rvKa3c58htCoPqXg1V5d14jhNXXmOV/zb3N27g3sb1XB9MgipdjVOoYlB1ZrBWr2E8j0OvHOYHv/9dTh4+jtRDgskJbGxT3migrck+X1OXLKNDzjwTM3oBdGkOVkecMOMZbJwQLbQAw/qtt/DAZ97Hlk/v5LpbNmAwPHJ1J/tffY09e/Zz6ewFCALqKybptSOe2/MS3L4Z369RN/X0QA3wfkHV0k9O95MTYlAsqGZhfM2EAFy1EU/PvckL8+9wV30tD028h5uDFXgiRDbGeB4TjQYnT5zi6a8/xeu7nMowmJpAE4uN3XWx6TdDkXd/QFnU46iSYcGeHXsCFve3eXuT5/lUFKL5Ofxmk3s/9RjbfuFD3P7BrTSak3STDvOteVSEeqPGB97/KA9uv58DBw6ye88+zp46Cxb8dSvx/QYqDob1uess66+AVVQ0pQrya6FD5lLTbI+HwTc1ErW82j7Da+2zvCdcyQPNjWxpXg/tmO999Ql2ffd5ooU2frPpJitKBtDGapmTq0hsZiImFVEpqRQkzVGqHbMAWmXhS9SS5jQKfactePD+v/1J/Ym/8XGW37yWiISkG9FqzYERjO8AV5wkRAsLBL7PYzt3sPWBLbx58C32v/Ayx+YvpQheBlojdR+nFqQZQD2AbozO99wO61MS+f3Xpz5kQBK7HWmo4czZO73LvBPNsOvqMeJ/sY+Lh07gLZvEn5hAbeLer0VpnKZZJKlQ6eXcuUqemhtk35RFTVABlw2AYV9/I2WzoymYiiy6qsGJOw2HZ85w16oaK5ZNkSBY47L6fX054mjlWC2ddodlfoMt2+6l9Z4JTk6/Tlz4FIHYonUfljewL56GJ99CPnYHsm2t25VzESQKniHbc2IG2Z8cFFNVFBes1UyAiHC+08JeuYK/bAoVRa0FNYMkXTKYRNFqTVlRwlfaz1KZqB9jggb7vuB8ZYjbSV/WwTE5ceQdThw8xpoNa3nk4Ye4f9sWlk1O0ul1ieLYmQMRasZHEU5Fs7x25RgHWqe5SodgZd1NpgKJhSAQrqurHr8Cf38X+h8PIq029l+9ivnoLfD5e+HRDRAYtxCxddGyyW08HdyOpCehr6FRgcD4JEGAxm3wcwqKcgZpSFBSZaczAaCOAaPjYKhZNNGsktsDpdUNwhAxhgsXLvPNr3+L51/Yw8MPb2Pb9vtYvmwK7SX0NOHN7gx7509ytHuero3xxKcuITZWNFE3Eaua6Pk2+qWX4XcOIOfnkDCEiSbECfaJI8gfvYU+thH5ha3IR26G1Q2Y60E3QTxTEnXm6z6cbxhMlxa3meb8iTGp49VCLr/vXKUACaRK75UzhIvBUKwdnJChVdOKNXXnIf2HUx8k+J6HTAZcvnyFJ7/xHXY9v4eHdz7E1Pab2Tv/LiejK1iUUAJqxtnkJLHOn69soPM97O++Av/iFZXjM+AH6EQdSdSdDBGkWXfs5HMn0edOwj1rkc/eAx+/HTZMovM9aMfZQmh+6qSIF1WWpj6UwAMvU51rJizINK1aUSul5dyRHe8DZFRuLZcOK4pPcgDXnW9BUbXiBz4SBly50tLvfucHsOE+mKhTE2d/bR+7I7C8DrFinziK/tN9woFpxGQTrxK7kEtSLaQmqT1ohG4S37iI/ubTyBf3w6fvgp+9W+SuVardBBaiNOVniodbJaOl1ebVmzII8xN1Js4IOj2PHLqEGoFikVWf8SoTBYvKoEqBmJhx1L8WlUZS0AcPBOdOeqKialyw4gc+Enh4BOCF2DgpiIs0StBvnUR/+xXk+dOIZ5SpppAoEmtFkDmA231cLo0ANQF6Zg75Jy/Cv3kN/sLtwmfuQbZf73bNXM+Zk9TSiiFL7KiWcq0W8A14AjNdODSDOTnrzJcvJS1sYWSjc4AiYHSMD1DRYYvPMMiWnPnJvSYpeNBBqKB9pGTVIlYzMVBWBlfz0KMX4F/tRV6+5D4qSaClSs1zk6CKaIrulAog6MwfFiH0VE0ArQi+/Cr8p0PoR29BPrcF3nsjUvOcn4jTqNaYgftUZ1PUM0jowUwHjl4WTswhcaIamJGK4eH8pRQwUWav7HgYWpnlymfdqzQY0p/RHJs5IBwFta56cYhAN0AvQTatQZ74i3C6BXvOoD88ie45B8dn0U4XwQi+70yB54q5XGQqA/CdR9lxAp6ohnWRRNFvHkX/+C3ksRvgF+4T+cmbVVfWYSFBerE7EZqmKz0Pme3Bm5dFjl9FuwmEBg29Po4dmudB8idFr1JUH+sY/ORfmyYmVwsq5A1RLt03OITZqud/Fkp4IMX6UQLrp+AvbYafvRtzuQOHZ9A9Z2HXadUDF2B6QSBRxYDvuV0qiAsttaj8UiBOK82aNXeInjupPPeu6pa18Iv3IR+/HdZNQs131+paOHoR3r6CdBK34DXPnUDbv+vUOQxyXpJxQf1SvxxgzBxzOqxFqAgrIwRx/ZktsM+pf9cqvUR/IrRKlaUO8VLgcRS6CXRi90mejzy4AfPYjdi/fh+cW0AOXVJemEb2TqOHZtCZhZQj8IXQV/VdZKxWszhKSR2pAM3QvfT6RfTXn4J//hL2F7fCxQXhjUtw/CosRBB4aM0MwI5WyFylIBfqp9NUc4UrKpkwF6RKMjG0ADIkWB3g/oJNKwVoubINh/myrHUG0LSKTpVB7afkcHeKiG0rgllHObCiCR+Ygo/ehnRj5Mwcdv9ZZPdZdM851aOXkPlun5xGaj7qmzTHmCbSE3UgreGDBKonZtH/7VmkXoPIOodb81PibRESvlRCnIle86Ws5aKVxRcgT3FrOW4pF4pK5TV1EH9nTi3vN7KF1NzSDarsHJQ1osZNXuYkY4UoAiKsEVg7iXzibuRTm5HZLrx7Bd1/Dl44A/vPq759FebbzkKIBzVPxTeoqmTMZt1DjC/ECqGXbtthlXSOZZAKSDnY5bpUOnMcGTfihQqmulAQgwGavgt+Eh1IRky6cNa9SSWdhP7K9DkfVVRMKpPDOd4+L9F38Pmz2LPQjVwSxRO4ZTWyaS3yma1wtYN564rw8jR21xl45Ty8OyvacUWe4nlo6IkYyfFFWtJYafb/dIgymrLPrVqVULNIBIldihNWxwVWFtAOkXeeKHOR8NoM3LpMmQyQxDqnlvRtvpKZg7wwVEscfxa528FL5UILBfH6OXEHU7UTo+04y8Rx71rlwfWYL9yHXlyAo5dF9k6ju8+gr11EzrRw6huPPsLSvl/SQbBbXR+2iEBQxhYK6KInoMAkZvxulTg6tU8GpBXB196C1XXhjpVw9wp0fROZ8FR7Cr3E0QgpYB9aBC3zz1rkb/I4QPKgOB2nKUpZdb4HrXSM9RB2bFQeuwGJE+T8PBy8iOy7gO45I7xxQfVcO6U+jRD4SGBQF6ApVmVsVZxWudehoLGyzt8ft4qqBUpVKi6QuRk1uMDpclfkhTPonrPIuiZsWgm3LodVNUc3TIYw202ti3VoqDBeLVxYcgyglKBHvuOBSMnI5hZELGgrcnofI8iyJvqBW+AjtyG9nurZFvLGJWTPWXTvWdVDl9CZdgoaPSHwHCQVSTdRBoUk66xAFW2ZdyaaK2MexwWlmzrHXeUNnMo4VZYq+KIa+M6GnV2A0/NQOwMbJoXEU/3grcLdK2Bl3Y18oYdEFhKb8jUDv5EnzwYjzLdgyDnufOZK0qBKczbeyCAGscBcz/FJnsDKSfjIMvhzt2O6CZyZxR44r+w6A/vOiR69rMx23GR7geKb8pTJsFsUhpGrSx7ainPSV8btFjE7VDVCxOs7Jh2j/Wd0m4cBV2TV9VOxkYofCLcsQ3ZsRHduRLavU33PMmgGjibpJmg3dnREfyebYXiMDCBaQTae6WeLmhktsSmFkp3U6ommGyD0oG7cIWon6LuzwqvTqrvOiH7/XbiwoBgjpR5HeZaywgtojJhArf0Nm1PGlRZgareI7ACNNF9BqZUTPaLPRi4FJSUewxhIrJpYRePIKRJW1FVvXy6yYyOyY4NyzxpkwyTa9N3CdRP3nYeHaayg/YzcgAPJCVN1QD2rFtma/hFXLfa7yTn9bGGMQeoeGgisrKOf/7bw71+FZh0SW5rwDJFKmbsEjRATqrUFbag/DKdkOLOQueGhhGQ+EMw3f9FsR+UbOCiKZ4g67fSNEVxJCF/uoS9NY/8lyIoGescqdNs6MTs2ODRz4xTS8J3J6CZueyR28LGSo8TKgqscUzNIVaUoWKSQKdECCybZtW0nhrnETW1SOOSlQj0ZBZXyv0sWRUFahL1umDpE8I2OMnK+s18HgAjR/AKQcOPDm/SxX3+cuXNX2PVbTzLz5knwaho0a2g7Qfeegb0nsf/awIomcsdKYfv16MPrkS1r4YYVyETgOKRODL1YM0osVVJnbsL0qWbJSJw8nFDJC160qEzoow+To6ZFlwZFq3NXi+aEtYKKHsQoOq40czhtL8ZxM9F8G0j0xofvZuevPc6WT+7ED10l49a/+kEOfOVZ3fulpzj/+nEgIJhsuA2YWHQ+QveeUd17SuRLBl3ZQO9YpeahDfDg9aJb1yAbJ6ARuoxZN3GqicQOUpG5tiLFOjAtBrnDFRUlLcOSivyl2sdWL5g/4o/NQAvBUFM9GSlF10wZB0K04Cb+hh138divfZwtn3oMP6zRarfotFqg4DdqPPY3foYHPv8RXv/a8+z54pOceekoYDRoNoXQQ31DFrW2IthzWuyed52CflUD7lwFD27APLwB3bwGvXEKqRkHdTsx0rOQiIgnODa1aPNHRlD5O65icYuIXBYt7V6cjhZThAi5DmJ5mFq2+al1Fc810EonnhseuZtH/9bj3PPJRwnCOgvtBTfxnrhIFrBJwtzcVbzA56HP/RRb//JPcOSPdrPrt77Fuz88qKD4jaazHzaFjUGYZbGYi9AXTwsvnsR+0aCrJ5BNq+CB9bBjI9yzWs2GKZJaQNSaV4mVwPgFMFXsv2gkJ4wdMOwqVBwXpZrkHBE4Z50gFw/E0iYwMkSTFM9jZklFhHihrZDwnp2b5eFfe1w3/fQj1MIG7YUFOr05jPHcBJaqZozvoQrzrRbG89j66Q+w6ZOPcuTJl9j9xW9x7KlXFU3w6xMinqQazXQo5QWZ7aLPnUSfO4HBV13doLs6wDy0UW/7nz7IzE11Ls9fBc8jlCCXUBEKlaep99bcj44m92TxHErlSVqKMGsQ/mh5k4/k6UREwCax3va+e3js7/wMt/7UfeqFNToLbeZ7c4jxMJ43pr1C6uwcW0mr1UI8YdPjO7nr8R0ce+YAe37rWxz945fUdnri1ZpqfE80SdwVbA5O+gapN7DWEs0vYGYsmx95kJ0//zg3bb6HWT/iYHCWV9vTnElaqlYxePjGy+U4dHBvNj0JzRDxAqHdc1BRtTzTWmAWy8HYwESNc8JWc7IUqaD+hj1Q/3gFntR/7h5t7rgREeM4FxGsjKpQUcrNsjLo7rufF+bnERFu++AD3PbBBzjx4kH2fPFbevjrLxLNz6oXNjGBL07NloqDE0s8N4cXhLrpU4+y89c/wc2PbsGiLHTbhOrxyNRNbJt8D8e7M7zanuat7iU6SSSIRyiuT6G1rveBNELBF/TQBfRPT6geOAfGH6OYGt0BrUoZV1WmunFY6T7WtTjCNPQ5t6LHS3te4dTJUxIGoay8bjUTzSZJYklsUtLjFEuEtASrlT59YFxtb9Rjxa3Xs/mTj3HH49uxFi4dOU1v7qogARiI2y08z9dNn97Jx/71r/K+X/8Uy268jvb8AnEcZ1KYnrqTs8af5J7GWu6sr2HCqzFvO8zFCyRxjIQB0gzh3Vn4g0Po1w/C+QW42IFLHXFigcpJ1oqipdT2iwf6HU16LzKiSnI3YnagmjC6SlILwhlJK8pCD/O5u5Wp0MFOhY233MBDOx/krns3MTU5Sa/bI7IJpc6IhZzxoA3iIAHah13WOjlLONkgIODc0RPs+9KTvPrlZ4haXdnys4/qjv/5cW548C5irGvqIeLEWfk0aklZ7ItHiKGV9HjXznFIZzj6znGSZ46hz59EW123GA0fdp+Ft64INW8gVR8y0VWlGVkkPI6KyBYgZrjJnJTT85rXTdY95LN3QTNw4hZjiHo9iGJWbVjLg49s457tW1nZnMJaS1eTfoCUSpp0KLWhucCo0NpZFbUQNmuEps7Zo28RzXa48cFNRMT0Wu1BmwIKUoUqFburUfMMk/VJurPz7P72szz3x88Sz7ShGaBGXIxR82HfeXjrsmQZNNBy3DSCqYkQCdXq/2J7c/90BBWxFKKtgjXO6eOcIguwVv0gQMKQmUtXeOqr3+Z5b5rN99/LfbKOG2srQZSOxi6rlaWSBySaYLJOgWlzvvQzHIztdXp0kw4rb78BYwytVstJ4T2vlLItdRHoCwasRYxhcmKS+fkFnv/mD3jhiWdonb6AaTaRyZpra5O4BcdaZJCtKc2RjKHJlhoHjO5eJWNlK/nVUNG8NNmq4vs+0mjSSRL2zr3L/t4Jbm+s4b7mRm6trWbCC1mwPSdekH6pg8mrmrUsqsxoDt8QdXrOaqU1CIWGWDKUs3bxhEBzYoIoitj/zB5e+Pr3ufD2KaRWc1L1xDruSXOkt2bX1iHkMNr8aE6EOrSXS9JEI4sn1YaS0SrFLtE5YmzAKFlrXYGEF5IQcbh9nsOdc2wIlnFv8wburK9lpd8gUUtPk3RwoghY1VIbavoJU80EGEYq5MMlk6OudVl9ooEiHN77Gs/9/vc49cYxCHz8qUmwrmK+3zJcGRSASFo+OxwIaylhL1T0RWOoZX55AVStpMGwMq7ATwsN/Aav2byIV3IHdcBU2lTGGqb1W9PRPGeuvMHz3ttsrl/PlokNXB8uRxQiTcTmWrHn9VCOz7T5OhHNF41qqUGRWkutUSfwAo4dfIvn/uAp3trzBojgTzRdkV9sB0Cyn4XL0dtayTbkA6aK5HWB1RtO6vvDSvYxyWcdRI1SqqdRweku03OakygWUqn9u+oPOTAB4LNgY/a0jvNy+xS31a5ja3MDN9dWUxOfnsYkmpDTO2opFNEMMaWzkbh0mOucFYbUgpDpE6fZ9Y0fcODZl9BejN9spHSITTW6UkA22RFXddRJYJCsYUeRl682S+VcgRly1iUfYLTYL1nLBSAyLOB1x18iC8fnlE0rIXSZLZce7JN6/fx1UU1k03NixMPzA1GsHmmf40j7LOvD5dzb3Mjd9bUs8+okqkSakKgVlQFBm7YKFRRMapY0UUzg0aw1mTl/ke8/8QP2f3830ew8XrOBNEOstaMsg/spVVJLzYeFGHn9LJycg8Abalo0WARdXJsy0gRhVdSM51plBCthgW8eh5fPwwNrhDtXKA0funFua0rOmItovhTFoSAFCDxnns7GC5ydPczu+Xe4u76ee5sbWedPIkBXo8wYWacABoSeTcQYoTE5xfzcHM8+8W32/vGfMn/+MmaigT856Wy8tYOexJLLONlcvrnmO0b1zSvCkRl1skUj1SKe8jMSqtIsi3BBufIAHcfpVcrsRNzOP92Cky1lXRPuuw42r4SVNZhNi+kSTXGkpLIbzVWySZY3UYRAfASPuSRiT+sdXlk4zS3hKu6d2MjN4SoaxqNnE2K1WXqxPjFB3Ovx0nf/VF78g6d15t1ppFHHn3ITb9Pk/6DIJecx+uSb77lHMky3lEMzMNMG3+Sx/9D8akmqPDBIWiFlG+mEB+3cqki3clZ7oHjJJX2cYhkuteGpE7D3HNy/Bm5djkyG0Aygs+AUWsaUmFFVtysl4+utgm8MqEeiliOdcxzpnOX6YBlbmhu5q7Ge5RK6z/WEI7te5YXf/66cOXgcwgB/2aTrKe3kJANlRH/H9xkcmzLRgUFmOnB0Bqbn3eDKssVRZUVSUaFxbfkAm+QYCK3w6AxnRKVooPrV0b5RAg/mesIzp2D3tDJv0V++H9m6ziXoZztO9u0WIh8QZyqGjEF1ckUCCQFlOp5n+uphXmy9w5bJjdxyucberzzFmy++BiLqT024rom5YmtJnb8UN5KDsHVPafVE3rysvHPVFXBkBRmLpB7zxUjlZzmVg1nRbmUGDLAmnHpKRH4Sso5Zpc7KBVQ9KhFRzBn0kVCC0utBLRQ+dBPy81uRD9wEE77qbBd6KuI53J/VW6tIX7ouWM04/5wqJY5i7KQHX30D/uAw3tplLnLNP7FGSnRG7hoEnqsjOH5F5M0ZWIghMIOa1kXVtlqw8lLqnJV7KXZtK5Ofs9H8fypTEX1m7sSAhdaixGQMNaFFZVoxo53XUDVrzgc8+Sb65Fuw7Xr4uc3w+B1w/aTqQoK0IwcnXD4568qRexhY1gXI6XI9fAmxfg1tNlLIPnh3NtmZwDYFeoFRxMDpOTh8SeRyx5XHOnMji+76Kko4H5JJGdmL53K9cjKPvcrdcQ9qdgdSirO1ImJQ+vWBIxjA4s9Jeo1mzZ2ll6fh5dPCP39J+eRdyF/aLGy6TkksMtdDNUE8V+fgRmVVKGp6+4VoKmTUQb43aVbX0P8DX1wN2MWOcOiScrblTukidr54N5rbdLnyVMkXrwyx6wbVK3HMkYFOt9wd1+ieFMZ6489ePvkwFHiwaJ60UGIqcLol/LNd6L95Ff3obSKfuRfdeYNSM+jVnhDbVA5opF8rNPR4NekTd/151Ky1Tb9zCTVPme8Jhy/CiauQWFmqnddcdC+Fzr9SaC6QByWDRRCb/tk+aF1k8Giw4kTbuHfBeOHPI7KKrHmrjJEGVD2pZakPmJKCnpQwFLoJvD4NXz0ML54SMUb0tuXK6rqTu0c242Qk17RbQh+OXoLDFx1lbPM9SsTVf1l1PP7L5+D8vMsle3INHfAk5w1lYOIKhJMM4s6iQbaI8UB/K03GeFUL4AM944c3gDwKmmQqieGi7VL/jiWK48duMXVoJAydvv/YDHzrKHz7mDNHN61A1k+CZ5BeksZdaXlj6KFvzqAHLyChN+hoErgibE61YN9Z4cRVdwv+4h05NadULqgtpcgxiRRbA5ab96aKBQ+0Y43/14k7s3mr4w1Ng1d7x8CvIH3zVOA0pBQLl07DtTycdESg12c+Q999n18Q/uQ4fO0IvHMVWTsBNy132aleAjFC3Ufevoy+fs7tduOqKLnUFvafF45eEqLEIZ4lTPxQrJnBV8n2ioiWpoVcvVtBFps4oTNf1e7Vf5vf/eUFcNVtSe+i8cJbELMthU5eUXQkeu3bffzk5zCuFFh0TWFiGMB8JLx8Cn7vkPDytJv021ajy0MR34MD59Cjl5CJGrR68NoFeO280HJVj5jRBItWP8inn6aWggnKWjPlGwDqEP+W5jX6nZrjBP0MSe9SGUV6lVnl0HvJWPlFxNTdaompyPaMmVEdUpHlEcSo5z7IqNlxeFMIQ3ehoxfgG0fg6XdEuhZuXQFXu7DvrCs13TctXGy7wgpv5DOotKijH4w012s314RDxo1x6ME6KYudiBgf7Be11/r35d0/agd7QGKCyV8WY37bPZBM/CVI75aAIoY/Uln08arDb+k7z3akEMNNq0U2r8E+f1KllRJmi2tnq8Q7Q0kPGdkzYxBpFVviZO9K3LzZI0m38SBcaFPxBO5RM+oeLhBO/lvE+5xbBIIf1bhfy1uHe0GOuUba4ZZOjNoECXznwK8piCpnmWQRT6b9J8iNUUOkFYYQGSvvjaKr+/LQcykL0G8fa0w49aQx5sPpgzzDpc1soVBhnLhiOJcxtPOW8MTyrPRIf4wTSjFmyJnS4qeW3lN6oDHuSRkCeKj9ZNJr/SFjni88TvujQGx7wSex9hmg5pzyUpCzUJ0dkuE9p2Of2Vme/NEPFLSL9RMffm14WxTJDi2lf/OFHlkHDs13rSR2ZgdRy19JJ99nzMOdF4l4MdDu2KT3e8ar3STCA+l4qtrbVwlJpbhbKoIbKaaVZaTdlSGxpWZFx8NzOd6+Vxdy5Z5Pj6hUjltKoq70/YoQg4QolxD+ou3NfqNPuI3bqt4STqYBIk26f2i88CrCoyDNdO2TEQHAAICNBUuaSntzlNaIBRt+yJ1mCDGHuWQJ+HhpT34qxlVV59c9rFPcQ9pAPFV9xsJPa29uz1Im/1rC1qzekXDqDg/+D0Q+nTsFcam+TBaLcbRqUjUr0GexJ6OwiHP5Ef1AFdQrlEcPkr7i55D1CRX+ke3OfikPYpY6sdfylV3Y95c9qEZ/Cfh4+uS9CmepS0H6Y+ayquAwTzsuwUFXPnBqkfePvI9i9kpVXwL+g+2ZL8PVK4WNeg07+1q/TJ5OhZXLvVr3YVX/IUE3K7IRtJle20s/od+oy+QZ4kJHWS08dcHm0Gg/FkrSf3slIrR8w8WVco5isWfiFK/h3m2LDxqQHuhlgeOqut9DX4yi+deqNuf/V19mCT7kv4cv/8cxf/JnMADJpS+19P1n+SX/Ga65lLnRkh/MWwD7P/bff+Vf/y/G5YdJmOrLuAAAAABJRU5ErkJggg==";

function makePixelIconDataUri(texPath) {
  const c = document.createElement('canvas');
  c.width = 16;
  c.height = 16;
  const ctx = c.getContext('2d');
  const p = String(texPath || '').toLowerCase();
  let primary = '#38bdf8', secondary = '#0284c7', border = '#0f172a';
  if (p.includes('emerald') || p.includes('grass') || p.includes('Note')) {
    primary = '#4ade80'; secondary = '#16a34a';
  } else if (p.includes('gold') || p.includes('helmet') || p.includes('blaze') || p.includes('bell') || p.includes('totem')) {
    primary = '#facc15'; secondary = '#ca8a04';
  } else if (p.includes('redstone') || p.includes('poppy') || p.includes('apple')) {
    primary = '#f87171'; secondary = '#dc2626';
  } else if (p.includes('command') || p.includes('crafting') || p.includes('chest') || p.includes('oak') || p.includes('planks')) {
    primary = '#d97706'; secondary = '#92400e';
  } else if (p.includes('disc') || p.includes('enchant') || p.includes('ender')) {
    primary = '#c084fc'; secondary = '#7e22ce';
  }
  ctx.fillStyle = border;
  ctx.fillRect(2, 2, 12, 12);
  ctx.fillStyle = secondary;
  ctx.fillRect(3, 3, 10, 10);
  ctx.fillStyle = primary;
  ctx.fillRect(4, 4, 6, 6);
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(5, 5, 2, 2);
  return c.toDataURL('image/png');
}

function hydrateAllMinecraftTextures() {
  document.querySelectorAll('img[data-gh-repo-asset]').forEach((img) => {
    img.src = SAWBYTE_LOGO_DATA_URI;
  });

  document.querySelectorAll('img[data-mc-tex]').forEach((img) => {
    const tex = img.getAttribute('data-mc-tex') || 'item/diamond.png';
    const fb = img.getAttribute('data-mc-fallback');
    const urls = [...getTextureUrls(tex), ...(fb ? getTextureUrls(fb) : []), makePixelIconDataUri(tex)];
    attachMultiSourceImage(img, urls);
  });
}

function initMcAssetRegistry() {
  document.querySelectorAll('.mc-pano-face[data-pano-face]').forEach((faceEl) => {
    const idx = faceEl.getAttribute('data-pano-face');
    const urls = getTextureUrls(`gui/title/background/panorama_${idx}.png`);
    const probe = new Image();
    let i = 0;
    const tryLoad = () => {
      if (i < urls.length) {
        const u = urls[i++];
        probe.onload = () => {
          faceEl.style.backgroundImage = `url('${u}')`;
        };
        probe.onerror = tryLoad;
        probe.src = u;
      }
    };
    tryLoad();
  });

  hydrateAllMinecraftTextures();

  // Preload 100% Real Minecraft Block, Leaf, Log, Water, Sun, Throne, Crack & Item Textures
  const blockAssets = {
    grass_side: { path: 'block/grass_block_side.png' },
    grass_top: { path: 'block/grass_block_top.png', tint: '#59c93c' },
    dirt: { path: 'block/dirt.png' },
    oak_log: { path: 'block/oak_log.png' },
    oak_log_top: { path: 'block/oak_log_top.png' },
    cherry_log: { path: 'block/cherry_log.png', fallback: 'block/oak_log.png' },
    cherry_log_top: { path: 'block/cherry_log_top.png', fallback: 'block/oak_log_top.png' },
    oak_leaves: { path: 'block/oak_leaves.png', tint: '#48b518' },
    azalea_leaves: { path: 'block/azalea_leaves.png' },
    flowering_azalea_leaves: { path: 'block/flowering_azalea_leaves.png' },
    cherry_leaves: { path: 'block/cherry_leaves.png', fallback: 'block/flowering_azalea_leaves.png' },
    water_still: { path: 'block/water_still.png', tint: '#3f76e4' },
    sun: { path: 'environment/sun.png' },
    stone: { path: 'block/stone.png' },
    deepslate: { path: 'block/deepslate.png' },
    cobblestone: { path: 'block/cobblestone.png' },
    coal_ore: { path: 'block/coal_ore.png' },
    iron_ore: { path: 'block/iron_ore.png' },
    gold_ore: { path: 'block/gold_ore.png' },
    redstone_ore: { path: 'block/redstone_ore.png' },
    emerald_ore: { path: 'block/emerald_ore.png' },
    diamond_ore: { path: 'block/diamond_ore.png' },
    deepslate_diamond_ore: { path: 'block/deepslate_diamond_ore.png' },
    gold_block: { path: 'block/gold_block.png' },
    red_wool: { path: 'block/red_wool.png' },
    obsidian: { path: 'block/obsidian.png' },
    nether_bricks: { path: 'block/nether_bricks.png' },
    diamond_block: { path: 'block/diamond_block.png' },
    emerald_block: { path: 'block/emerald_block.png' },
    crafting_table: { path: 'block/crafting_table_front.png' },
    crafting_table_top: { path: 'block/crafting_table_top.png' },
    crafting_table_side: { path: 'block/crafting_table_side.png' },
    poppy: { path: 'block/poppy.png' },
    crown: { path: 'item/golden_helmet.png' },
    pickaxe: { path: 'item/diamond_pickaxe.png' },
    sword: { path: 'item/netherite_sword.png' },
    potato: { path: 'item/baked_potato.png' },
    fishing_rod: { path: 'item/fishing_rod.png' },
    totem: { path: 'item/totem_of_undying.png' },
    diamond: { path: 'item/diamond.png' },
    pigstep_disc: { path: 'item/music_disc_pigstep.png' }
  };

  for (const [k, spec] of Object.entries(blockAssets)) {
    const urls = getTextureUrls(spec.path);
    if (spec.fallback) {
      urls.push(...getTextureUrls(spec.fallback));
    }
    loadCachedTexture(k, urls, spec.tint || null);
  }

  for (let c = 0; c <= 9; c++) {
    loadCachedTexture(`crack_${c}`, getTextureUrls(`block/destroy_stage_${c}.png`));
  }

  // Load Uploaded Transparent Skins for Technoblade, Steve, and Alex
  loadCachedTexture('skin_techno', [SKIN_DATA_URIS.techno]);
  loadCachedTexture('skin_techno_seated', [SKIN_DATA_URIS.techno_seated]);
  loadCachedTexture('skin_steve', [SKIN_DATA_URIS.steve]);
  loadCachedTexture('skin_alex', [SKIN_DATA_URIS.alex]);
  loadCachedTexture('head_techno', [SKIN_DATA_URIS.techno]);
  loadCachedTexture('techno_full', [SKIN_DATA_URIS.techno]);

  // Also ensure Tribute Card images fallback cleanly to the uploaded Technoblade skin
  document.querySelectorAll('.tribute-img').forEach((img) => {
    img.setAttribute('data-fallback-skin', SKIN_DATA_URIS.techno);
  });

  // Fetch Minecraft Asset Index JSON from GitHub for SHA-1 Sound Resolution
  const indexUrls = [
    'https://raw.githubusercontent.com/catppuccin/minecraft/main/resource-packs/Catppuccin%20UI/lang/indexes/1.18.2.json',
    'https://cdn.jsdelivr.net/gh/catppuccin/minecraft@main/resource-packs/Catppuccin%20UI/lang/indexes/1.18.2.json'
  ];

  (async () => {
    for (const url of indexUrls) {
      try {
        const res = await fetch(url);
        if (!res.ok) continue;
        const data = await res.json();
        if (data && data.objects) {
          for (const [key, val] of Object.entries(data.objects)) {
            if (key.startsWith('minecraft/sounds/') && val && val.hash) {
              SOUND_SHA1_CACHE[key] = val.hash;
            }
          }
          if (bgMusicAudio && !bgMusicPlaying && !userPausedMusic) {
            loadDiscIntoPlayer(currentDiscIndex, true);
          }
          break;
        }
      } catch (_) {}
    }
  })();
}

function initOptionalScreenshots() {
  document.querySelectorAll('img[data-optional-screenshot]').forEach((img) => {
    const path = img.getAttribute('data-optional-screenshot');
    if (!path) return;
    const probe = new Image();
    probe.onload = () => {
      img.src = path;
      img.style.display = 'block';
    };
    probe.onerror = () => {};
    probe.src = path;
  });
}

// ==============================================================================
// 2. Slower, Natural Login Intro + Singleplayer Misclick + Technoblade Vortex
// ==============================================================================

function initMinecraftIntroSequence() {
  const overlay = document.getElementById('mc-intro-overlay');
  const cursor = document.getElementById('mc-virtual-cursor');
  const replayBtn = document.getElementById('replay-intro-btn');
  const skipBtn = document.getElementById('mc-intro-skip-btn');
  if (!overlay) return;

  let panoramaActive = true;
  let introTimers = [];
  startPanoramaRenderer(() => panoramaActive);

  function clearAllIntroTimers() {
    introTimers.forEach((id) => clearTimeout(id));
    introTimers = [];
  }

  function scheduleIntroStep(fn, delayMs) {
    const id = setTimeout(fn, delayMs);
    introTimers.push(id);
  }

  const KEY_SHORT_COUNT = 'sawbyte_intro_short_visits';
  const KEY_LAST_TS = 'sawbyte_intro_last_ts';
  const LONG_WAIT_MS = 30 * 60 * 1000;

  let shortVisits = 0;
  let lastTs = 0;
  try {
    shortVisits = parseInt(localStorage.getItem(KEY_SHORT_COUNT) || '99', 10);
    lastTs = parseInt(localStorage.getItem(KEY_LAST_TS) || '0', 10);
  } catch (_) {
    shortVisits = 99;
  }

  const now = Date.now();
  const waitedLongTime = lastTs === 0 || (now - lastTs > LONG_WAIT_MS);
  const shouldPlayFullIntro = shortVisits >= 5 || waitedLongTime;

  try {
    localStorage.setItem(KEY_LAST_TS, String(now));
    if (shouldPlayFullIntro) {
      localStorage.setItem(KEY_SHORT_COUNT, '1');
    } else {
      localStorage.setItem(KEY_SHORT_COUNT, String(shortVisits + 1));
    }
  } catch (_) {}

  if (replayBtn) {
    replayBtn.addEventListener('click', () => {
      try {
        localStorage.setItem(KEY_SHORT_COUNT, '1');
      } catch (_) {}
      runIntroSequence(true);
    });
  }

  if (skipBtn) {
    skipBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      clearAllIntroTimers();
      finishIntro();
    });
  }

  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && overlay.style.display !== 'none') {
      clearAllIntroTimers();
      finishIntro();
    }
  });

  if (overlay) { overlay.style.display = 'none'; } document.body.classList.remove('intro-cursor-locked'); panoramaActive = false;

  function showIntroScreen(screenId) {
    document.querySelectorAll('.mc-intro-screen').forEach((s) => {
      s.classList.toggle('active', s.id === screenId);
    });
  }

  function moveCursorToElement(el, offsetX = 0, offsetY = 0) {
    if (!cursor || !el) return;
    const rect = el.getBoundingClientRect();
    cursor.style.left = `${rect.left + rect.width / 2 + offsetX}px`;
    cursor.style.top = `${rect.top + rect.height / 2 + offsetY}px`;
  }

  function clickButtonElement(btnEl, cb) {
    if (!btnEl) {
      if (cb) cb();
      return;
    }
    btnEl.classList.add('hovered');
    scheduleIntroStep(() => {
      if (cursor) cursor.classList.add('clicking');
      btnEl.classList.add('pressed');
      playMcClick(1.0);
      scheduleIntroStep(() => {
        if (cursor) cursor.classList.remove('clicking');
        btnEl.classList.remove('pressed');
        btnEl.classList.remove('hovered');
        if (cb) cb();
      }, 210);
    }, 340);
  }

  /**
   * Loading Terrain Phase + Swirling Minecraft Blocks & Items Forming Technoblade!
   */
  function runLoadingTerrainWithTechnoVortex(durationMs, onDone) {
    showIntroScreen('intro-screen-loading');
    if (cursor) cursor.style.display = 'none';

    const labelEl = document.getElementById('mc-loading-stage-text');
    const fillEl = document.getElementById('mc-terrain-progress-fill');
    const grayBox = document.getElementById('mc-chunk-gray-box');
    const vCanvas = document.getElementById('mc-techno-vortex-canvas');
    const vCtx = vCanvas ? vCanvas.getContext('2d') : null;

    if (labelEl) labelEl.textContent = 'Connecting to server...';
    if (fillEl) fillEl.style.width = '5%';
    if (grayBox) {
      grayBox.style.width = '24px';
      grayBox.style.height = '24px';
    }

    // Create 42 swirling Minecraft items/blocks that converge into Technoblade at the center!
    const itemKeys = ['crown', 'sword', 'potato', 'poppy', 'totem', 'diamond', 'pickaxe', 'gold_ore', 'diamond_ore', 'emerald_ore', 'pigstep_disc'];
    const particles = [];
    for (let i = 0; i < 42; i++) {
      const angle = (i / 42) * Math.PI * 2;
      const radius = 110 + (i % 3) * 35;
      particles.push({
        baseAngle: angle,
        startRadius: radius,
        texKey: itemKeys[i % itemKeys.length],
        targetX: ((i % 7) - 3) * 14,
        targetY: (Math.floor(i / 7) - 2.5) * 14
      });
    }

    const start = performance.now();
    function step(ts) {
      const elapsed = ts - start;
      const pct = Math.min(1, elapsed / durationMs);

      if (pct > 0.16 && pct < 0.72 && labelEl) {
        labelEl.textContent = 'Loading terrain...';
      } else if (pct >= 0.72 && labelEl) {
        labelEl.textContent = 'Assembling Technoblade tribute...';
      }

      if (fillEl) fillEl.style.width = `${Math.floor(pct * 100)}%`;
      if (grayBox) {
        const size = Math.floor(24 + pct * 54);
        grayBox.style.width = `${size}px`;
        grayBox.style.height = `${size}px`;
      }

      // Render Swirling Block/Item Vortex Forming Technoblade
      if (vCtx && vCanvas) {
        const cw = vCanvas.width;
        const ch = vCanvas.height;
        const cx = cw / 2;
        const cy = ch / 2 - 6;
        vCtx.clearRect(0, 0, cw, ch);
        vCtx.imageSmoothingEnabled = false;

        // Smooth ease-in-out convergence factor (0 = wide swirling ring, 1 = assembled Technoblade)
        const converge = Math.min(1, Math.max(0, (pct - 0.15) / 0.65));
        const easeConv = converge * converge * (3 - 2 * converge);

        for (const p of particles) {
          const spinAngle = p.baseAngle + elapsed * 0.0055 * (1.2 - easeConv * 0.9);
          const curRadius = p.startRadius * (1 - easeConv);
          const orbitX = Math.cos(spinAngle) * curRadius;
          const orbitY = Math.sin(spinAngle) * curRadius * 0.68;

          const px = cx + orbitX + p.targetX * easeConv;
          const py = cy + orbitY + p.targetY * easeConv;

          const img = WORLD_TEX_IMAGES[p.texKey];
          const sz = Math.floor(18 * (1 - easeConv * 0.35));
          if (isTexReady(img)) {
            vCtx.globalAlpha = 1 - easeConv * 0.55;
            vCtx.drawImage(img, Math.floor(px - sz / 2), Math.floor(py - sz / 2), sz, sz);
          }
        }
        vCtx.globalAlpha = 1;

        // As the blocks converge in the center, reveal Technoblade's Crowned Avatar & Golden Halo!
        if (easeConv > 0.25) {
          const revealAlpha = Math.min(1, (easeConv - 0.25) / 0.55);
          vCtx.save();
          vCtx.globalAlpha = revealAlpha;

          // Golden Crown Aura
          const grad = vCtx.createRadialGradient(cx, cy, 8, cx, cy, 74);
          grad.addColorStop(0, 'rgba(255, 170, 0, 0.55)');
          grad.addColorStop(1, 'rgba(255, 85, 85, 0)');
          vCtx.fillStyle = grad;
          vCtx.beginPath();
          vCtx.arc(cx, cy, 74, 0, Math.PI * 2);
          vCtx.fill();

          const technoImg = WORLD_TEX_IMAGES.skin_techno || WORLD_TEX_IMAGES.head_techno;
          if (isTexReady(technoImg)) {
            const hSize = Math.floor(116 * (0.75 + 0.25 * revealAlpha));
            const wSize = Math.floor(hSize * (78 / 150));
            vCtx.drawImage(technoImg, Math.floor(cx - wSize / 2), Math.floor(cy - hSize / 2 - 4), wSize, hSize);
          }

          vCtx.fillStyle = '#FFFF55';
          vCtx.font = '13px MinecraftOfficial, monospace';
          vCtx.textAlign = 'center';
          vCtx.fillText('TECHNOBLADE NEVER DIES [o7]', cx, ch - 10);
          vCtx.restore();
        }
      }

      if (pct < 1) {
        requestAnimationFrame(step);
      } else {
        playMcLevelUp();
        if (onDone) onDone();
      }
    }
    requestAnimationFrame(step);
  }

  function finishIntro() {
    worldIntroActive = false;
    overlay.classList.add('fade-out');
    document.body.classList.remove('intro-cursor-locked');
    setTimeout(() => {
      overlay.style.display = 'none';
      panoramaActive = false;
    }, 380);
  }

  function runIntroSequence(fullMode) {
    clearAllIntroTimers();
    panoramaActive = true;
    worldIntroActive = true;
    overlay.style.display = 'flex';
    overlay.classList.remove('fade-out');
    document.body.classList.add('intro-cursor-locked');

    const emptyMsg = document.getElementById('mc-mp-empty-msg');
    const serverRow = document.getElementById('mc-server-row-sawbyte');
    const joinBtn = document.getElementById('mc-btn-join-server');
    const editBtn = document.getElementById('mc-btn-edit-server');
    const deleteBtn = document.getElementById('mc-btn-delete-server');
    const nameInput = document.getElementById('mc-input-server-name');
    const addrInput = document.getElementById('mc-input-server-addr');

    if (emptyMsg) emptyMsg.style.display = 'block';
    if (serverRow) {
      serverRow.style.display = 'none';
      serverRow.classList.remove('selected');
    }
    if (joinBtn) joinBtn.classList.add('disabled');
    if (editBtn) editBtn.classList.add('disabled');
    if (deleteBtn) deleteBtn.classList.add('disabled');
    if (nameInput) {
      nameInput.textContent = 'Minecraft Server';
      nameInput.classList.remove('focused');
    }
    if (addrInput) {
      addrInput.textContent = '';
      addrInput.classList.remove('focused');
    }

    // Repeat visits 2..5: play the 2.3-second Loading Terrain + Technoblade Vortex
    if (!fullMode) {
      runLoadingTerrainWithTechnoVortex(2300, finishIntro);
      return;
    }

    // Full First-Visit Sequence (Comfortable, readable human pace!)
    showIntroScreen('intro-screen-title');
    if (cursor) {
      cursor.style.display = 'block';
      cursor.style.left = `${Math.floor(window.innerWidth * 0.75)}px`;
      cursor.style.top = `${Math.floor(window.innerHeight * 0.68)}px`;
    }

    const spBtn = document.getElementById('mc-btn-singleplayer');
    const spCancelBtn = document.getElementById('mc-btn-sp-cancel');
    const mpBtn = document.getElementById('mc-btn-multiplayer');
    const addServerBtn = document.getElementById('mc-btn-add-server');
    const addDoneBtn = document.getElementById('mc-btn-add-done');

    // Step 1: Move cursor to Singleplayer by mistake and click it!
    scheduleIntroStep(() => {
      if (spBtn) spBtn.classList.add('hovered');
      moveCursorToElement(spBtn);
    }, 650);

    scheduleIntroStep(() => {
      clickButtonElement(spBtn, () => {
        // Step 2: Inside Singleplayer Select World menu -> realize mistake -> move to Cancel
        showIntroScreen('intro-screen-singleplayer');

        scheduleIntroStep(() => {
          if (spCancelBtn) spCancelBtn.classList.add('hovered');
          moveCursorToElement(spCancelBtn);
        }, 750);

        scheduleIntroStep(() => {
          clickButtonElement(spCancelBtn, () => {
            // Step 3: Back on Title Screen -> move to Multiplayer and click!
            showIntroScreen('intro-screen-title');

            scheduleIntroStep(() => {
              if (mpBtn) mpBtn.classList.add('hovered');
              moveCursorToElement(mpBtn);
            }, 600);

            scheduleIntroStep(() => {
              clickButtonElement(mpBtn, () => {
                // Step 4: Open Play Multiplayer -> move to Add Server and click!
                showIntroScreen('intro-screen-multiplayer');

                scheduleIntroStep(() => {
                  if (addServerBtn) addServerBtn.classList.add('hovered');
                  moveCursorToElement(addServerBtn);
                }, 650);

                scheduleIntroStep(() => {
                  clickButtonElement(addServerBtn, () => {
                    // Step 5: Inside Edit Server Info -> paste URL into Server Address
                    showIntroScreen('intro-screen-add-server');

                    scheduleIntroStep(() => {
                      moveCursorToElement(addrInput);
                    }, 550);

                    scheduleIntroStep(() => {
                      if (cursor) cursor.classList.add('clicking');
                      playMcClick(1.15);
                      if (addrInput) {
                        addrInput.classList.add('focused');
                        addrInput.textContent = '_';
                      }
                      scheduleIntroStep(() => {
                        if (cursor) cursor.classList.remove('clicking');
                        if (addrInput) {
                          addrInput.textContent = 'https://5ivesaw.github.io/Sawbyte_';
                        }
                        playMcClick(1.3);
                      }, 280);
                    }, 1200);

                    // Step 6: Move up to Server Name and enter "Sawbyte"
                    scheduleIntroStep(() => {
                      moveCursorToElement(nameInput);
                    }, 1850);

                    scheduleIntroStep(() => {
                      if (cursor) cursor.classList.add('clicking');
                      playMcClick(1.15);
                      if (addrInput) {
                        addrInput.classList.remove('focused');
                        addrInput.textContent = 'https://5ivesaw.github.io/Sawbyte';
                      }
                      if (nameInput) {
                        nameInput.classList.add('focused');
                        nameInput.textContent = 'Sawbyte_';
                      }
                      scheduleIntroStep(() => {
                        if (cursor) cursor.classList.remove('clicking');
                      }, 180);
                    }, 2500);

                    // Step 7: Move down to Done and click
                    scheduleIntroStep(() => {
                      if (addDoneBtn) addDoneBtn.classList.add('hovered');
                      moveCursorToElement(addDoneBtn);
                    }, 3050);

                    scheduleIntroStep(() => {
                      clickButtonElement(addDoneBtn, () => {
                        // Step 8: Back on Play Multiplayer -> select Sawbyte -> click Join Server
                        if (emptyMsg) emptyMsg.style.display = 'none';
                        if (serverRow) serverRow.style.display = 'flex';
                        showIntroScreen('intro-screen-multiplayer');

                        scheduleIntroStep(() => {
                          moveCursorToElement(serverRow);
                        }, 550);

                        scheduleIntroStep(() => {
                          if (cursor) cursor.classList.add('clicking');
                          playMcClick(1.05);
                          if (serverRow) serverRow.classList.add('selected');
                          if (joinBtn) joinBtn.classList.remove('disabled');
                          if (editBtn) editBtn.classList.remove('disabled');
                          if (deleteBtn) deleteBtn.classList.remove('disabled');
                          scheduleIntroStep(() => {
                            if (cursor) cursor.classList.remove('clicking');
                          }, 180);
                        }, 1200);

                        scheduleIntroStep(() => {
                          if (joinBtn) joinBtn.classList.add('hovered');
                          moveCursorToElement(joinBtn);
                        }, 1750);

                        scheduleIntroStep(() => {
                          clickButtonElement(joinBtn, () => {
                            // Step 9: Loading terrain + Swirling Block Vortex Forming Technoblade!
                            runLoadingTerrainWithTechnoVortex(2850, finishIntro);
                          });
                        }, 2450);
                      });
                    }, 3750);
                  });
                }, 1350);
              });
            }, 1300);
          });
        }, 1500);
      });
    }, 1450);
  }
}

function startPanoramaRenderer() {
  // Real panorama_0..5.png textures are rendered on the hardware-accelerated 3D cubemap faces (.mc-pano-face).
}

// ==============================================================================
// 3. 15-Second Rotating Weapon & Tool Cursor (Desktop Mouse + Mobile Touch Tap)
// ==============================================================================

function initWeaponToolCursor() {
  const cursorEl = document.getElementById('mc-weapon-cursor');
  const itemWrap = document.getElementById('mc-cursor-item-wrap');
  const itemImg = document.getElementById('mc-cursor-item-img');
  const cdWrap = document.getElementById('mc-cursor-cooldown-wrap');
  const cdFill = document.getElementById('mc-cursor-cooldown-fill');

  const hudImg = document.getElementById('hud-tool-img');
  const hudName = document.getElementById('hud-tool-name');
  const hudTimer = document.getElementById('hud-tool-timer');
  const hudCdText = document.getElementById('hud-tool-cd-text');
  const hudCdOverlay = document.getElementById('hud-tool-cd-overlay');
  const swapBtn = document.getElementById('tool-swap-btn');

  let hideTouchCursorTimer = null;

  function equipRandomTool(announce = false) {
    const candidates = CURSOR_ARSENAL.filter((t) => t.id !== currentTool.id);
    currentTool = candidates[Math.floor(Math.random() * candidates.length)] || CURSOR_ARSENAL[0];
    toolCountdownSecs = 15;

    const urls = getTextureUrls(currentTool.tex);
    if (currentTool.fallbackTex) {
      urls.push(...getTextureUrls(currentTool.fallbackTex));
    }
    if (itemImg) attachMultiSourceImage(itemImg, urls);
    if (hudImg) attachMultiSourceImage(hudImg, urls);
    if (hudName) hudName.textContent = currentTool.name;
    if (hudCdText) hudCdText.textContent = `${(currentTool.cooldownMs / 1000).toFixed(2)}s`;

    if (announce) {
      playMinecraftOgg('random/pop.ogg', 0.35, 1.15, () => synthClickFallback(1.3));
    }
  }

  equipRandomTool(false);

  setInterval(() => {
    if (document.body.classList.contains('intro-cursor-locked')) return;
    toolCountdownSecs--;
    if (toolCountdownSecs <= 0) {
      equipRandomTool(true);
    }
    if (hudTimer) hudTimer.textContent = `${toolCountdownSecs}s`;
  }, 1000);

  if (swapBtn) {
    swapBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      equipRandomTool(true);
      showAdvancementToast('Equipped New Tool!', `${currentTool.name} (CD ${(currentTool.cooldownMs / 1000).toFixed(2)}s)`, currentTool.tex);
    });
  }

  window.swapCursorToolNow = () => equipRandomTool(true);

  window.addEventListener('pointermove', (e) => {
    if (!cursorEl || e.pointerType === 'touch') return;
    cursorEl.classList.add('cursor-visible');
    cursorEl.style.left = `${e.clientX}px`;
    cursorEl.style.top = `${e.clientY}px`;
  });

  window.addEventListener('pointerdown', (e) => {
    if (document.body.classList.contains('intro-cursor-locked')) return;
    const isInteractive = document.body.getAttribute('data-site-mode') === 'interactive';
    const isMcCursor = document.body.getAttribute('data-cursor-mode') === 'mc';
    const clickedBgCanvas = e.target && e.target.id === 'bg-canvas';
    if (!isMcCursor && !(isInteractive && clickedBgCanvas)) return;
    if (e.target && e.target.closest('a, button, input, select, textarea, .navbar, .jukebox-dock, .tool-hud-dock, .site-mode-selector-bar, .disc-maker-studio, .terminal-card, .bench-card, .download-card, .metric-card, .screenshot-slot-card, .tribute-frame')) {
      return;
    }

    // On mobile/touch, show the weapon cursor right where the user tapped!
    if (cursorEl) {
      cursorEl.classList.add('cursor-visible');
      cursorEl.style.left = `${e.clientX}px`;
      cursorEl.style.top = `${e.clientY}px`;
      if (e.pointerType === 'touch') {
        if (hideTouchCursorTimer) clearTimeout(hideTouchCursorTimer);
        hideTouchCursorTimer = setTimeout(() => {
          cursorEl.classList.remove('cursor-visible');
        }, 1200);
      }
    }

    const now = performance.now();
    const elapsed = now - lastAttackTs;
    lastChargeRatio = Math.min(1, elapsed / currentTool.cooldownMs);
    lastAttackTs = now;

    if (itemWrap) {
      itemWrap.classList.remove('swinging');
      void itemWrap.offsetWidth;
      itemWrap.classList.add('swinging');
    }

    if (lastChargeRatio >= 0.82) {
      playMinecraftOgg(currentTool.sound, 0.42, 0.95 + Math.random() * 0.1, () => {
        synthToolSoundFallback(currentTool.fallbackFreq, 1.0);
      });
      if (currentTool.id === 'golden_sword' && Math.random() < 0.35) {
        setTimeout(() => playMcPigSound(), 90);
      }
      if (['sword', 'axe', 'all'].includes(currentTool.affinity)) {
        spawnSweepSlash(e.clientX, e.clientY);
      }
    } else {
      playMinecraftOgg('entity/player/attack/weak1.ogg', 0.28, 1.2, () => {
        synthToolSoundFallback(currentTool.fallbackFreq * 0.7, 0.4);
      });
    }

    animateToolCooldown(currentTool.cooldownMs);

    // Check if the click/tap hit a 2.5D block or character in the background scene!
    const tag = e.target?.tagName?.toLowerCase() || '';
    if (!['a', 'button', 'input', 'select', 'textarea'].includes(tag) && !e.target?.closest('a, button, input, select, .seq-cell, .tribute-figure-card, .jukebox-dock, .tool-hud-dock')) {
      if (typeof window.hitShader25DWorldAt === 'function') {
        window.hitShader25DWorldAt(e.clientX, e.clientY);
      }
    }
  });

  let cdRafId = null;
  function animateToolCooldown(durationMs) {
    if (cdRafId) cancelAnimationFrame(cdRafId);
    if (cdWrap) cdWrap.classList.add('active');
    const start = performance.now();

    function step(ts) {
      const pct = Math.min(1, (ts - start) / durationMs);
      if (cdFill) cdFill.style.width = `${Math.floor(pct * 100)}%`;
      if (hudCdOverlay) hudCdOverlay.style.height = `${Math.floor((1 - pct) * 100)}%`;

      if (pct < 1) {
        cdRafId = requestAnimationFrame(step);
      } else {
        if (cdWrap) cdWrap.classList.remove('active');
        if (hudCdOverlay) hudCdOverlay.style.height = '0%';
      }
    }
    cdRafId = requestAnimationFrame(step);
  }
}

function spawnSweepSlash(x, y) {
  const slash = document.createElement('div');
  slash.className = 'mc-sweep-slash';
  slash.style.left = `${x}px`;
  slash.style.top = `${y}px`;
  document.body.appendChild(slash);
  setTimeout(() => {
    if (slash.parentNode) slash.parentNode.removeChild(slash);
  }, 250);
}

function synthToolSoundFallback(freq = 280, power = 1.0) {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = power > 0.8 ? 'sawtooth' : 'triangle';
  osc.frequency.setValueAtTime(freq, now);
  osc.frequency.exponentialRampToValueAtTime(freq * 0.35, now + 0.09);
  gain.gain.setValueAtTime(0.08 * power, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.095);
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + 0.1);
}

// ==============================================================================
// 4. Full-Screen Interactive 2.5D Shader-Lit Minecraft World Background
//    - Volumetric golden-hour sunbeams, shader water river, swaying trees,
//      breakable 2.5D blocks & ores (respawn after 2s), and animated
//      Steve, Alex, and Technoblade!
// ==============================================================================

const WORLD_BLOCK_DEFS = {
  grass: { name: 'Grass Block', tex: 'grass_side', topTex: 'grass_top', sideTex: 'grass_side', hp: 14, cat: 'shovel', sound: 'dig/grass1.ogg' },
  dirt: { name: 'Dirt', tex: 'dirt', topTex: 'dirt', sideTex: 'dirt', hp: 12, cat: 'shovel', sound: 'dig/grass2.ogg' },
  oak_log: { name: 'Oak Log', tex: 'oak_log', topTex: 'oak_log_top', sideTex: 'oak_log', hp: 18, cat: 'axe', sound: 'dig/wood1.ogg', dropTex: 'block/oak_log.png', dropLabel: '+1 Oak Log' },
  cherry_log: { name: 'Cherry Log', tex: 'cherry_log', topTex: 'cherry_log_top', sideTex: 'cherry_log', hp: 18, cat: 'axe', sound: 'dig/wood1.ogg', dropTex: 'block/cherry_log.png', dropLabel: '+1 Cherry Log' },
  oak_leaves: { name: 'Oak Leaves', tex: 'oak_leaves', topTex: 'oak_leaves', sideTex: 'oak_leaves', hp: 6, cat: 'hoe', sound: 'dig/grass3.ogg', isLeaf: true },
  azalea_leaves: { name: 'Azalea Leaves', tex: 'azalea_leaves', topTex: 'azalea_leaves', sideTex: 'azalea_leaves', hp: 6, cat: 'hoe', sound: 'dig/grass3.ogg', isLeaf: true },
  flowering_azalea_leaves: { name: 'Flowering Azalea Leaves', tex: 'flowering_azalea_leaves', topTex: 'flowering_azalea_leaves', sideTex: 'flowering_azalea_leaves', hp: 6, cat: 'hoe', sound: 'dig/grass3.ogg', isLeaf: true },
  cherry_leaves: { name: 'Cherry Leaves', tex: 'cherry_leaves', topTex: 'cherry_leaves', sideTex: 'cherry_leaves', hp: 6, cat: 'hoe', sound: 'dig/grass3.ogg', isLeaf: true },
  stone: { name: 'Stone', tex: 'stone', topTex: 'stone', sideTex: 'stone', hp: 20, cat: 'pickaxe', sound: 'dig/stone1.ogg' },
  deepslate: { name: 'Deepslate', tex: 'deepslate', topTex: 'deepslate', sideTex: 'deepslate', hp: 24, cat: 'pickaxe', sound: 'dig/stone2.ogg' },
  gold_ore: { name: 'Gold Ore', tex: 'gold_ore', topTex: 'gold_ore', sideTex: 'gold_ore', hp: 26, cat: 'pickaxe', sound: 'dig/stone3.ogg', dropTex: 'item/raw_gold.png', dropLabel: '+1 Raw Gold', glow: 'rgba(255, 204, 21, 0.28)' },
  redstone_ore: { name: 'Redstone Ore', tex: 'redstone_ore', topTex: 'redstone_ore', sideTex: 'redstone_ore', hp: 26, cat: 'pickaxe', sound: 'dig/stone4.ogg', dropTex: 'item/redstone.png', dropLabel: '+4 Redstone', glow: 'rgba(239, 68, 68, 0.30)' },
  emerald_ore: { name: 'Emerald Ore', tex: 'emerald_ore', topTex: 'emerald_ore', sideTex: 'emerald_ore', hp: 28, cat: 'pickaxe', sound: 'dig/stone1.ogg', dropTex: 'item/emerald.png', dropLabel: '+1 Emerald', glow: 'rgba(34, 197, 94, 0.30)' },
  diamond_ore: { name: 'Diamond Ore', tex: 'diamond_ore', topTex: 'diamond_ore', sideTex: 'diamond_ore', hp: 30, cat: 'pickaxe', sound: 'dig/stone2.ogg', dropTex: 'item/diamond.png', dropLabel: '+1 Diamond', glow: 'rgba(56, 189, 248, 0.36)' },
  gold_block: { name: 'Block of Gold', tex: 'gold_block', topTex: 'gold_block', sideTex: 'gold_block', hp: 40, cat: 'pickaxe', sound: 'dig/stone1.ogg', glow: 'rgba(255, 215, 0, 0.22)' },
  red_wool: { name: 'Royal Red Wool', tex: 'red_wool', topTex: 'red_wool', sideTex: 'red_wool', hp: 12, cat: 'sword', sound: 'dig/cloth1.ogg' },
  obsidian: { name: 'Obsidian', tex: 'obsidian', topTex: 'obsidian', sideTex: 'obsidian', hp: 60, cat: 'pickaxe', sound: 'dig/stone4.ogg' },
  nether_bricks: { name: 'Nether Bricks', tex: 'nether_bricks', topTex: 'nether_bricks', sideTex: 'nether_bricks', hp: 30, cat: 'pickaxe', sound: 'dig/stone2.ogg' },
  diamond_block: { name: 'Block of Diamond', tex: 'diamond_block', topTex: 'diamond_block', sideTex: 'diamond_block', hp: 40, cat: 'pickaxe', sound: 'dig/stone1.ogg', glow: 'rgba(56, 189, 248, 0.28)' },
  emerald_block: { name: 'Block of Emerald', tex: 'emerald_block', topTex: 'emerald_block', sideTex: 'emerald_block', hp: 40, cat: 'pickaxe', sound: 'dig/stone1.ogg' },
  crafting_table: { name: 'Crafting Table', tex: 'crafting_table', topTex: 'crafting_table_top', sideTex: 'crafting_table_side', hp: 18, cat: 'axe', sound: 'dig/wood2.ogg' }
};

/**
 * Pre-bakes a 2.5D Isometric Block Sprite using 100% Real Minecraft Textures
 * (Top Face + Shaded Right Face + Crisp Front Face). Never draws fake flat color boxes!
 */
function getOrBakeBlockSprite(type, size, bevel = 8) {
  const cacheKey = `${type}_${size}_${bevel}`;
  if (BAKED_BLOCK_SPRITES[cacheKey]) return BAKED_BLOCK_SPRITES[cacheKey];

  const def = WORLD_BLOCK_DEFS[type] || WORLD_BLOCK_DEFS.stone;
  const frontImg = WORLD_TEX_IMAGES[def.tex];
  if (!isTexReady(frontImg)) return null;

  const topImg = isTexReady(WORLD_TEX_IMAGES[def.topTex]) ? WORLD_TEX_IMAGES[def.topTex] : frontImg;
  const sideImg = isTexReady(WORLD_TEX_IMAGES[def.sideTex]) ? WORLD_TEX_IMAGES[def.sideTex] : frontImg;

  const c = document.createElement('canvas');
  c.width = size + bevel;
  c.height = size + bevel;
  const bctx = c.getContext('2d');
  bctx.imageSmoothingEnabled = false;

  // 1. Top 2.5D Bevel Face (Real Top Texture skewed 45 deg)
  const topScratch = document.createElement('canvas');
  topScratch.width = size;
  topScratch.height = bevel;
  const tctx = topScratch.getContext('2d');
  tctx.imageSmoothingEnabled = false;
  tctx.drawImage(topImg, 0, 0, size, bevel);
  tctx.globalCompositeOperation = 'source-atop';
  tctx.fillStyle = 'rgba(255, 255, 255, 0.12)';
  tctx.fillRect(0, 0, size, bevel);

  bctx.save();
  bctx.setTransform(1, 0, -1, 1, bevel, 0);
  bctx.drawImage(topScratch, 0, 0);
  bctx.restore();

  // 2. Right 2.5D Bevel Face (Real Side Texture skewed -45 deg & shaded for 3D depth)
  const sideScratch = document.createElement('canvas');
  sideScratch.width = bevel;
  sideScratch.height = size;
  const sctx = sideScratch.getContext('2d');
  sctx.imageSmoothingEnabled = false;
  sctx.drawImage(sideImg, 0, 0, bevel, size);
  sctx.globalCompositeOperation = 'source-atop';
  sctx.fillStyle = 'rgba(0, 0, 0, 0.28)';
  sctx.fillRect(0, 0, bevel, size);

  bctx.save();
  bctx.setTransform(1, -1, 0, 1, size, bevel);
  bctx.drawImage(sideScratch, 0, 0);
  bctx.restore();

  // 3. Front Block Face (Real Front Texture)
  bctx.drawImage(frontImg, 0, bevel, size, size);

  BAKED_BLOCK_SPRITES[cacheKey] = c;
  return c;
}

let canvasDebrisPool = [];

function spawnBlockDebris(x, y, texKey, count = 6) {
  const maxParticles = 32;
  for (let i = 0; i < count; i++) {
    if (canvasDebrisPool.length >= maxParticles) canvasDebrisPool.shift();
    canvasDebrisPool.push({
      x,
      y,
      vx: (Math.random() - 0.5) * 4.8,
      vy: -(Math.random() * 3.8 + 1.2),
      texKey: texKey || 'stone',
      sx: Math.floor(Math.random() * 10),
      sy: Math.floor(Math.random() * 10),
      life: 1.0
    });
  }
}

function initShader25DWorldCanvas() {
  const canvas = document.getElementById('bg-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d', { alpha: false });

  let width = (canvas.width = window.innerWidth);
  let height = (canvas.height = window.innerHeight);

  // Offscreen Buffers for Zero-Lag 60-120 FPS Rendering
  const skyBufferCanvas = document.createElement('canvas');
  const terrainBufferCanvas = document.createElement('canvas');
  let terrainDirty = true;

  window.markWorldTerrainDirty = () => {
    terrainDirty = true;
  };

  let worldBlocks = [];
  let waterBlocks = [];
  let damagedOrScalingBlocks = [];
  let characters = [];
  let cherryPetals = [];
  const depthBevel = 8;

  function rebuildSkyBuffer() {
    skyBufferCanvas.width = width;
    skyBufferCanvas.height = height;
    const sctx = skyBufferCanvas.getContext('2d');

    // 1. Complementary Shader Sky Gradient
    const skyGrad = sctx.createLinearGradient(0, 0, 0, height * 0.78);
    skyGrad.addColorStop(0, '#0a1124');
    skyGrad.addColorStop(0.45, '#1b2b52');
    skyGrad.addColorStop(0.75, '#4b365f');
    skyGrad.addColorStop(1, '#9c5243');
    sctx.fillStyle = skyGrad;
    sctx.fillRect(0, 0, width, height);

    // 2. Shader Sun Bloom & Volumetric Rays (pre-baked once!)
    const sunX = width * 0.75;
    const sunY = height * 0.18;
    const sunGlow = sctx.createRadialGradient(sunX, sunY, 8, sunX, sunY, 220);
    sunGlow.addColorStop(0, 'rgba(255, 235, 160, 0.48)');
    sunGlow.addColorStop(0.45, 'rgba(255, 165, 85, 0.16)');
    sunGlow.addColorStop(1, 'rgba(255, 120, 60, 0)');
    sctx.fillStyle = sunGlow;
    sctx.fillRect(0, 0, width, height);

    sctx.save();
    sctx.globalCompositeOperation = 'screen';
    for (let r = 0; r < 4; r++) {
      sctx.fillStyle = 'rgba(255, 210, 140, 0.032)';
      sctx.beginPath();
      sctx.moveTo(sunX - 16 + r * 12, sunY);
      sctx.lineTo(sunX - 360 + r * 150, height);
      sctx.lineTo(sunX - 240 + r * 150, height);
      sctx.closePath();
      sctx.fill();
    }
    sctx.restore();
  }

  function rebuildTerrainBuffer() {
    terrainBufferCanvas.width = width;
    terrainBufferCanvas.height = height;
    const tctx = terrainBufferCanvas.getContext('2d');
    tctx.clearRect(0, 0, width, height);
    tctx.imageSmoothingEnabled = false;

    damagedOrScalingBlocks = [];

    for (const b of worldBlocks) {
      if (b.broken) {
        tctx.fillStyle = 'rgba(5, 6, 10, 0.52)';
        tctx.fillRect(b.x, b.y, b.size, b.size);
        continue;
      }

      if (b.hp < b.maxHp || b.respawnScale < 1 || (b.hitShake && b.hitShake > 0.2)) {
        damagedOrScalingBlocks.push(b);
        continue;
      }

      if (b.def.glow) {
        tctx.fillStyle = b.def.glow;
        tctx.fillRect(b.x - 3, b.y - 3, b.size + 6, b.size + 6);
      }

      const sprite = getOrBakeBlockSprite(b.type, b.size, depthBevel);
      if (sprite) {
        tctx.drawImage(sprite, b.x, b.y - depthBevel);
      }
    }

    terrainDirty = false;
  }

  function buildSceneGrid() {
    width = canvas.width = window.innerWidth;
    height = canvas.height = window.innerHeight;
    worldBlocks = [];
    waterBlocks = [];

    const bSize = width < 640 ? 38 : 46;
    const cols = Math.ceil(width / bSize) + 1;
    const baseGroundRowY = Math.floor(height * 0.73);
    const throneCol = Math.max(5, Math.min(cols - 4, Math.floor(cols * 0.78)));

    const surfaceYByCol = [];

    for (let c = 0; c < cols; c++) {
      const normX = c / Math.max(1, cols - 1);
      const isRiver = normX > 0.38 && normX < 0.56;
      const isThronePlateau = Math.abs(c - throneCol) <= 2;

      let hillOffset = 0;
      if (isThronePlateau) {
        hillOffset = 2;
      } else if (normX <= 0.38) {
        hillOffset = Math.round(Math.sin(normX * Math.PI * 2.2) * 2);
      } else if (normX >= 0.56) {
        hillOffset = Math.round(Math.sin((normX - 0.55) * Math.PI * 2.0) * 1.5) + 1;
      }

      const topY = isRiver ? baseGroundRowY + bSize : baseGroundRowY - hillOffset * bSize;
      surfaceYByCol[c] = topY;

      // Animated Minecraft water_still.png blocks in river basin
      if (isRiver) {
        waterBlocks.push({
          col: c,
          x: c * bSize,
          y: baseGroundRowY,
          size: bSize
        });
      }

      // Surface Block (Obsidian/Gold Dais under the Royal Throne, Grass on hills)
      let surfaceType = isRiver ? 'dirt' : 'grass';
      let surfaceBreakable = true;
      if (isThronePlateau) {
        surfaceType = Math.abs(c - throneCol) === 2 ? 'obsidian' : (c === throneCol ? 'red_wool' : 'gold_block');
        surfaceBreakable = false;
      }
      addBlock(c * bSize, topY, bSize, surfaceType, surfaceBreakable);

      // Subsurface Dirt + Stone + Exposed Ores
      const oreChoices = ['stone', 'stone', 'deepslate', 'gold_ore', 'redstone_ore', 'emerald_ore', 'diamond_ore'];
      for (let depth = 1; depth <= 6; depth++) {
        const by = topY + depth * bSize;
        if (by > height + bSize) break;
        let bType = 'dirt';
        if (isThronePlateau && depth === 1) {
          bType = 'nether_bricks';
        } else if (depth === 2) {
          bType = 'stone';
        } else if (depth >= 3) {
          bType = oreChoices[(c * 7 + depth * 3) % oreChoices.length];
        }
        addBlock(c * bSize, by, bSize, bType, !isThronePlateau);
      }

      // Crafting Table on left hill
      if (c === Math.floor(cols * 0.24)) {
        addBlock(c * bSize, topY - bSize, bSize, 'crafting_table', true);
      }
    }

    // Build Voxel Trees Using 100% Real Minecraft Leaf & Log Textures
    const treeSpecs = [
      { col: Math.floor(cols * 0.11), log: 'oak_log', leafPrimary: 'oak_leaves', leafAccent: 'azalea_leaves' },
      { col: Math.floor(cols * 0.63), log: 'cherry_log', leafPrimary: 'cherry_leaves', leafAccent: 'flowering_azalea_leaves' },
      { col: Math.min(cols - 2, throneCol + 4), log: 'oak_log', leafPrimary: 'azalea_leaves', leafAccent: 'flowering_azalea_leaves' }
    ];

    for (const tree of treeSpecs) {
      if (Math.abs(tree.col - throneCol) <= 2) continue;
      const tTopY = surfaceYByCol[tree.col] || baseGroundRowY;
      for (let th = 1; th <= 3; th++) {
        addBlock(tree.col * bSize, tTopY - th * bSize, bSize, tree.log, true);
      }
      for (let lx = -2; lx <= 2; lx++) {
        for (let ly = 3; ly <= 5; ly++) {
          if (ly === 5 && Math.abs(lx) === 2) continue;
          const lType = (Math.abs(lx) + ly) % 2 === 0 ? tree.leafPrimary : tree.leafAccent;
          addBlock((tree.col + lx) * bSize, tTopY - ly * bSize, bSize, lType, true);
        }
      }
    }

    // Construct The Blood God's Grand Royal Throne on the Right Hill Plateau (Unbreakable)
    const throneSurfaceY = surfaceYByCol[throneCol];
    // Backrest Pillars & Royal Cushion
    addBlock((throneCol - 1) * bSize, throneSurfaceY - bSize, bSize, 'gold_block', false);
    addBlock(throneCol * bSize, throneSurfaceY - bSize, bSize, 'red_wool', false);
    addBlock((throneCol + 1) * bSize, throneSurfaceY - bSize, bSize, 'gold_block', false);

    addBlock((throneCol - 1) * bSize, throneSurfaceY - 2 * bSize, bSize, 'emerald_block', false);
    addBlock(throneCol * bSize, throneSurfaceY - 2 * bSize, bSize, 'red_wool', false);
    addBlock((throneCol + 1) * bSize, throneSurfaceY - 2 * bSize, bSize, 'emerald_block', false);

    addBlock((throneCol - 1) * bSize, throneSurfaceY - 3 * bSize, bSize, 'gold_block', false);
    addBlock(throneCol * bSize, throneSurfaceY - 3 * bSize, bSize, 'gold_block', false);
    addBlock((throneCol + 1) * bSize, throneSurfaceY - 3 * bSize, bSize, 'gold_block', false);

    addBlock(throneCol * bSize, throneSurfaceY - 4 * bSize, bSize, 'diamond_block', false);

    // Initialize Steve & Alex (Animated Walkers) + Technoblade (Seated Blood God King on His Throne!)
    characters = [
      {
        id: 'steve',
        name: 'Steve',
        skinKey: 'skin_steve',
        heldItem: 'pickaxe',
        isSeatedKing: false,
        colFloat: cols * 0.16,
        minCol: cols * 0.05,
        maxCol: cols * 0.33,
        dir: 1,
        speed: 0.68,
        actionTimer: 0,
        bubbleText: 'Mining deepslate diamonds!',
        jumpVel: 0,
        jumpY: 0
      },
      {
        id: 'alex',
        name: 'Alex',
        skinKey: 'skin_alex',
        heldItem: 'fishing_rod',
        isSeatedKing: false,
        colFloat: cols * 0.34,
        minCol: cols * 0.22,
        maxCol: cols * 0.38,
        dir: 1,
        speed: 0.52,
        actionTimer: 0,
        bubbleText: 'Fishing by the river',
        jumpVel: 0,
        jumpY: 0
      },
      {
        id: 'techno',
        name: 'Technoblade [Blood God]',
        skinKey: 'skin_techno_seated',
        standingSkinKey: 'skin_techno',
        heldItem: 'sword',
        isSeatedKing: true,
        colFloat: throneCol + 0.5,
        throneCol,
        throneSeatY: throneSurfaceY - bSize,
        dir: 1,
        speed: 0,
        actionTimer: 0,
        bubbleText: 'Blood for the Blood God!',
        jumpVel: 0,
        jumpY: 0
      }
    ];

    characters.forEach((ch) => {
      ch.surfaceYByCol = surfaceYByCol;
      ch.bSize = bSize;
    });

    cherryPetals = Array.from({ length: 24 }, () => ({
      x: Math.random() * width,
      y: Math.random() * height * 0.78,
      vx: (Math.random() - 0.3) * 0.4,
      vy: Math.random() * 0.32 + 0.14,
      size: Math.random() > 0.5 ? 4 : 3,
      isFirefly: Math.random() > 0.45
    }));

    rebuildSkyBuffer();
    terrainDirty = true;
  }

  function addBlock(x, y, size, type, breakable = true) {
    const def = WORLD_BLOCK_DEFS[type] || WORLD_BLOCK_DEFS.stone;
    worldBlocks.push({
      x,
      y,
      size,
      type,
      def,
      hp: def.hp,
      maxHp: def.hp,
      broken: false,
      breakable,
      respawnScale: 1
    });
  }

  buildSceneGrid();

  let resizeTimer = null;
  window.addEventListener('resize', () => {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(buildSceneGrid, 120);
  });

  // Hit-test background blocks or characters when user clicks/taps
  window.hitShader25DWorldAt = function (clientX, clientY) {
    for (const ch of characters) {
      if (ch.screenX && ch.screenY) {
        if (Math.abs(clientX - ch.screenX) < 38 && Math.abs(clientY - (ch.screenY - 36)) < 48) {
          ch.actionTimer = 65;
          if (ch.isSeatedKing) {
            const quotes = [
              'Blood for the Blood God!',
              'Technoblade Never Dies!',
              'Sit down, mortal. [o7]',
              '+500,000,000 Potatoes!'
            ];
            ch.bubbleText = quotes[Math.floor(Math.random() * quotes.length)];
            playMcPigSound();
          } else {
            ch.jumpVel = -6.8;
            ch.bubbleText = ch.id === 'steve' ? 'Found Deepslate Diamonds!' : 'Caught Enchanted Cod!';
            playMcXpChime();
          }
          return;
        }
      }
    }

    for (let i = worldBlocks.length - 1; i >= 0; i--) {
      const b = worldBlocks[i];
      if (!b.breakable || b.broken) continue;
      if (clientX >= b.x && clientX <= b.x + b.size && clientY >= b.y - depthBevel && clientY <= b.y + b.size) {
        let dmg = (currentTool.baseDamage || 8) * (0.4 + 0.6 * lastChargeRatio);
        if (currentTool.affinity === b.def.cat || currentTool.affinity === 'all') {
          dmg *= 2.3;
        }
        b.hp -= dmg;
        b.hitShake = 5;
        terrainDirty = true;

        spawnBlockDebris(clientX, clientY, b.def.tex, 5);

        if (b.hp > 0) {
          playMinecraftOgg(b.def.sound, 0.32, 0.92 + Math.random() * 0.16);
        } else {
          b.broken = true;
          b.hitShake = 0;
          terrainDirty = true;
          totalMinedBlocks++;
          const minedHud = document.getElementById('hud-mined-count');
          if (minedHud) minedHud.textContent = totalMinedBlocks.toLocaleString();

          playMinecraftOgg(b.def.sound, 0.5, 0.82);
          spawnBlockDebris(clientX, clientY, b.def.tex, 8);

          if (b.def.dropTex) {
            playMcXpChime();
            spawnFloatingOreDrop(clientX, clientY, b.def.dropTex, b.def.dropLabel || `+1 ${b.def.name}`);
          }

          setTimeout(() => {
            b.broken = false;
            b.hp = b.maxHp;
            b.respawnScale = 0.25;
            terrainDirty = true;
            playMinecraftOgg('random/pop.ogg', 0.2, 1.3);
          }, 2000);
        }
        break;
      }
    }
  };

  /**
   * Articulated Skin Renderer for Steve & Alex using their Uploaded Skins
   */
  function drawArticulatedWalker(ch, cx, cy, ts) {
    const skinImg = WORLD_TEX_IMAGES[ch.skinKey];
    if (!isTexReady(skinImg)) return;

    const sw = skinImg.width || skinImg.naturalWidth;
    const sh = skinImg.height || skinImg.naturalHeight;
    const scale = 0.44;
    const dw = Math.round(sw * scale);
    const dh = Math.round(sh * scale);

    const stride = Math.sin(ts * 0.0085 * ch.speed * 1.45);
    const legAngle = stride * 0.32;
    const bobY = Math.abs(stride) * 2.2;
    const armAngle = ch.actionTimer > 0
      ? Math.sin(ts * 0.032) * 0.65
      : -stride * 0.28;

    ctx.save();
    ctx.translate(cx, Math.floor(cy - bobY));
    if (ch.dir < 0) {
      ctx.scale(-1, 1);
    }

    // 1. Back Leg (sy: 98..sh, sx: 0..sw*0.52)
    const legSy = Math.floor(sh * 0.65);
    const legSh = sh - legSy;
    const midX = Math.floor(sw * 0.5);
    const legTopDy = -Math.round(legSh * scale);

    ctx.save();
    ctx.translate(-2, legTopDy + 2);
    ctx.rotate(legAngle);
    ctx.drawImage(
      skinImg,
      0, legSy, midX, legSh,
      -Math.round(midX * scale * 0.8), 0, Math.round(midX * scale), Math.round(legSh * scale)
    );
    ctx.restore();

    // 2. Front Leg (sy: 98..sh, sx: sw*0.48..sw)
    ctx.save();
    ctx.translate(2, legTopDy + 2);
    ctx.rotate(-legAngle);
    ctx.drawImage(
      skinImg,
      midX - 2, legSy, sw - (midX - 2), legSh,
      -4, 0, Math.round((sw - (midX - 2)) * scale), Math.round(legSh * scale)
    );
    ctx.restore();

    // 3. Torso + Back Arm (sy: 44..102, sx: 16..sw)
    const torsoSy = Math.floor(sh * 0.30);
    const torsoSh = Math.floor(sh * 0.38);
    const armSplitX = Math.floor(sw * 0.28);
    ctx.drawImage(
      skinImg,
      armSplitX, torsoSy, sw - armSplitX, torsoSh,
      Math.round((armSplitX - sw * 0.5) * scale), -dh + Math.round(torsoSy * scale),
      Math.round((sw - armSplitX) * scale), Math.round(torsoSh * scale)
    );

    // 4. Head (sy: 0..48, sx: 0..sw) with subtle look nod
    const headSh = Math.floor(sh * 0.33);
    ctx.save();
    ctx.translate(0, -dh + Math.round(headSh * scale));
    ctx.rotate(Math.sin(ts * 0.003 + (ch.id === 'steve' ? 0 : 1.5)) * 0.05);
    ctx.drawImage(
      skinImg,
      0, 0, sw, headSh,
      -Math.round(dw * 0.5), -Math.round(headSh * scale), dw, Math.round(headSh * scale)
    );
    ctx.restore();

    // 5. Front Swinging Arm (sy: 46..105, sx: 0..armSplitX+4) + Held Official Minecraft Tool
    ctx.save();
    const shoulderX = -Math.round(dw * 0.22);
    const shoulderY = -dh + Math.round(sh * 0.34 * scale);
    ctx.translate(shoulderX, shoulderY);
    ctx.rotate(armAngle);
    ctx.drawImage(
      skinImg,
      0, torsoSy, armSplitX + 4, torsoSh,
      -Math.round(armSplitX * scale * 0.5), 0, Math.round((armSplitX + 4) * scale), Math.round(torsoSh * scale)
    );

    const itemImg = WORLD_TEX_IMAGES[ch.heldItem];
    if (isTexReady(itemImg)) {
      ctx.drawImage(itemImg, -2, Math.round(torsoSh * scale * 0.55), 18, 18);
    }
    ctx.restore();

    ctx.restore();
  }

  /**
   * Renders Technoblade Seated Majestically on His Royal Throne as the Blood God King
   */
  function drawSeatedKingTechnoblade(ch, ts) {
    const cx = Math.floor(ch.colFloat * ch.bSize);
    const cy = Math.floor(ch.throneSeatY + 8);
    ch.screenX = cx;
    ch.screenY = cy;

    const breatheY = Math.sin(ts * 0.0022) * 1.2;

    // Subtle Royal Golden Aura behind the Throne Seat
    const seatedImg = WORLD_TEX_IMAGES.skin_techno_seated || WORLD_TEX_IMAGES.skin_techno;
    if (isTexReady(seatedImg)) {
      const sw = seatedImg.width || seatedImg.naturalWidth;
      const sh = seatedImg.height || seatedImg.naturalHeight;
      const scale = 0.52;
      const dw = Math.round(sw * scale);
      const dh = Math.round(sh * scale);

      ctx.drawImage(
        seatedImg,
        Math.floor(cx - dw * 0.48),
        Math.floor(cy - dh + 14 + breatheY),
        dw,
        dh
      );
    }

    // Netherite Sword planted point-down beside the Blood God's right golden armrest
    const swordImg = WORLD_TEX_IMAGES.sword;
    if (isTexReady(swordImg)) {
      ctx.save();
      ctx.translate(cx - 26, cy - 18 + breatheY * 0.5);
      ctx.rotate((135 * Math.PI) / 180);
      ctx.drawImage(swordImg, -12, -12, 24, 24);
      ctx.restore();
    }

    // Royal Nametag above the Throne
    if (ch.actionTimer > 0) ch.actionTimer--;
    ctx.font = '11px MinecraftOfficial, monospace';
    ctx.textAlign = 'center';
    const tagText = ch.actionTimer > 0 ? `${ch.name}: ${ch.bubbleText}` : ch.name;
    const textWidth = ctx.measureText(tagText).width + 12;
    const tagY = Math.floor(ch.throneSeatY - ch.bSize * 3.35);
    ctx.fillStyle = 'rgba(0, 0, 0, 0.68)';
    ctx.fillRect(Math.floor(cx - textWidth / 2), tagY - 12, Math.floor(textWidth), 16);
    ctx.fillStyle = '#FFAA00';
    ctx.fillText(tagText, cx, tagY);
  }

  // Zero-Lag 60-120 FPS Render Loop
  function renderWorld(ts) {
    // Pause heavy background rendering when tab is hidden or while full-screen intro overlay is opaque
    if (document.hidden || (worldIntroActive && !terrainDirty)) {
      requestAnimationFrame(renderWorld);
      return;
    }

    if (terrainDirty) {
      rebuildTerrainBuffer();
    }

    ctx.imageSmoothingEnabled = false;

    // 1. Single Blit of Cached Sky, Sun Glow & Volumetric Rays
    ctx.drawImage(skyBufferCanvas, 0, 0);

    // 2. Real Minecraft Sun Texture (environment/sun.png)
    const sunImg = WORLD_TEX_IMAGES.sun;
    if (isTexReady(sunImg)) {
      ctx.drawImage(sunImg, Math.floor(width * 0.75 - 28), Math.floor(height * 0.18 - 28), 56, 56);
    }

    // 3. Parallax Drifting Clouds
    ctx.fillStyle = 'rgba(255, 230, 215, 0.20)';
    for (let c = 0; c < 5; c++) {
      const cx = ((c * 310 + ts * 0.012 * (c % 2 === 0 ? 1 : 0.6)) % (width + 240)) - 120;
      const cy = 54 + (c % 3) * 38;
      ctx.fillRect(Math.floor(cx), cy, 110, 16);
      ctx.fillRect(Math.floor(cx + 20), cy - 8, 70, 8);
    }

    // 4. Animated Official Minecraft Water River (block/water_still.png 32-frame strip)
    const waterImg = WORLD_TEX_IMAGES.water_still;
    const waterFrame = Math.floor(ts / 110) % 32;
    for (const wb of waterBlocks) {
      const waveY = Math.round(Math.sin(ts * 0.0035 + wb.col * 0.8) * 2);
      if (isTexReady(waterImg)) {
        const fw = waterImg.width || 16;
        const fh = Math.min(fw, (waterImg.height || 512) - waterFrame * fw);
        ctx.globalAlpha = 0.86;
        ctx.drawImage(waterImg, 0, waterFrame * fw, fw, fh, wb.x, wb.y + waveY, wb.size, wb.size - waveY);
        ctx.globalAlpha = 1.0;
      }
    }

    // 5. Single Hardware-Accelerated Blit of Static 2.5D World Terrain, Trees & Royal Throne!
    ctx.drawImage(terrainBufferCanvas, 0, 0);

    // 6. Render Only Active Damaged/Scaling Blocks & Official destroy_stage_0..9.png Cracks
    let needsTerrainRefresh = false;
    for (const b of damagedOrScalingBlocks) {
      if (b.broken) continue;

      if (b.respawnScale < 1) {
        b.respawnScale = Math.min(1, b.respawnScale + 0.1);
        if (b.respawnScale >= 1 && b.hp >= b.maxHp) needsTerrainRefresh = true;
      }
      const shakeX = b.hitShake ? (Math.random() - 0.5) * b.hitShake : 0;
      if (b.hitShake > 0) {
        b.hitShake *= 0.72;
        if (b.hitShake < 0.2) {
          b.hitShake = 0;
          if (b.hp >= b.maxHp && b.respawnScale >= 1) needsTerrainRefresh = true;
        }
      }

      const drawX = Math.floor(b.x + shakeX);
      const drawY = Math.floor(b.y);
      const sz = Math.floor(b.size * b.respawnScale);

      const sprite = getOrBakeBlockSprite(b.type, sz, depthBevel);
      if (sprite) {
        ctx.drawImage(sprite, drawX, drawY - depthBevel);
      }

      if (b.hp < b.maxHp) {
        const dmgFrac = 1 - b.hp / b.maxHp;
        const stage = Math.min(9, Math.max(0, Math.floor(dmgFrac * 10)));
        const crackImg = WORLD_TEX_IMAGES[`crack_${stage}`];
        if (isTexReady(crackImg)) {
          ctx.drawImage(crackImg, drawX, drawY, sz, sz);
        }
      }
    }
    if (needsTerrainRefresh) terrainDirty = true;

    // 7. Render Seated King Technoblade on His Throne + Animated Steve & Alex
    for (const ch of characters) {
      if (ch.isSeatedKing) {
        drawSeatedKingTechnoblade(ch, ts);
        continue;
      }

      ch.colFloat += ch.dir * ch.speed * 0.015;
      if (ch.colFloat >= ch.maxCol) {
        ch.colFloat = ch.maxCol;
        ch.dir = -1;
      } else if (ch.colFloat <= ch.minCol) {
        ch.colFloat = ch.minCol;
        ch.dir = 1;
      }

      if (ch.jumpVel !== 0 || ch.jumpY < 0) {
        ch.jumpY += ch.jumpVel;
        ch.jumpVel += 0.46;
        if (ch.jumpY >= 0) {
          ch.jumpY = 0;
          ch.jumpVel = 0;
        }
      } else if (Math.random() < 0.0025) {
        ch.jumpVel = -4.8;
      }

      const colIdx = Math.max(0, Math.min(ch.surfaceYByCol.length - 1, Math.floor(ch.colFloat)));
      const groundY = ch.surfaceYByCol[colIdx] || height * 0.73;
      const cx = Math.floor(ch.colFloat * ch.bSize);
      const cy = Math.floor(groundY + ch.jumpY);
      ch.screenX = cx;
      ch.screenY = cy;

      if (ch.actionTimer > 0) ch.actionTimer--;

      ctx.fillStyle = 'rgba(0, 0, 0, 0.38)';
      ctx.fillRect(cx - 12, groundY - 2, 24, 4);

      drawArticulatedWalker(ch, cx, cy, ts);

      ctx.font = '11px MinecraftOfficial, monospace';
      ctx.textAlign = 'center';
      const tagText = ch.actionTimer > 0 ? `${ch.name}: ${ch.bubbleText}` : ch.name;
      const textWidth = ctx.measureText(tagText).width + 10;
      ctx.fillStyle = 'rgba(0, 0, 0, 0.58)';
      ctx.fillRect(Math.floor(cx - textWidth / 2), cy - 78, Math.floor(textWidth), 15);
      ctx.fillStyle = '#FFFFFF';
      ctx.fillText(tagText, cx, cy - 67);
    }

    // 8. Canvas-Native Block Break Debris Particles (Real Block Texture Crops, Zero DOM Lag)
    for (let i = canvasDebrisPool.length - 1; i >= 0; i--) {
      const dp = canvasDebrisPool[i];
      dp.x += dp.vx;
      dp.y += dp.vy;
      dp.vy += 0.28;
      dp.life -= 0.035;
      if (dp.life <= 0) {
        canvasDebrisPool.splice(i, 1);
        continue;
      }
      const texImg = WORLD_TEX_IMAGES[dp.texKey];
      if (isTexReady(texImg)) {
        ctx.globalAlpha = dp.life;
        ctx.drawImage(texImg, dp.sx, dp.sy, 6, 6, Math.floor(dp.x), Math.floor(dp.y), 7, 7);
        ctx.globalAlpha = 1.0;
      }
    }

    // 9. Lightweight Ambient Petals & Fireflies
    for (const p of cherryPetals) {
      p.x += p.vx;
      p.y += p.vy;
      if (p.y > height * 0.85) {
        p.y = -8;
        p.x = Math.random() * width;
      }
      if (p.x < 0) p.x = width;
      if (p.x > width) p.x = 0;

      ctx.fillStyle = p.isFirefly ? 'rgba(255, 230, 110, 0.65)' : 'rgba(244, 165, 198, 0.65)';
      ctx.fillRect(Math.floor(p.x), Math.floor(p.y), p.size, p.size);
    }

    requestAnimationFrame(renderWorld);
  }

  requestAnimationFrame(renderWorld);
}

function spawnFloatingOreDrop(x, y, dropTex, label) {
  const el = document.createElement('div');
  el.className = 'mc-floating-drop';
  el.style.left = `${x}px`;
  el.style.top = `${y - 10}px`;

  const img = document.createElement('img');
  img.className = 'mc-png mc-png-sm';
  attachMultiSourceImage(img, getTextureUrls(dropTex));

  const span = document.createElement('span');
  span.textContent = label;

  el.appendChild(img);
  el.appendChild(span);
  document.body.appendChild(el);

  setTimeout(() => {
    if (el.parentNode) el.parentNode.removeChild(el);
  }, 1150);
}

// ==============================================================================
// 5. 100% True Non-Hardcoded Stochastic disc_maker Composer & .OGG Disc Exporter
//    - Supports Total Random All-Minecraft-Sounds Mode (Mobs, Blocks, World, Items
//      & Dynamic Asset Index .ogg sounds — NOT restricted to Note Blocks!)
//    - Uses Bjorklund's Euclidean Algorithm E(k, 16, rotation) for rhythms
//    - Uses Markov Melodic Step-Walk + Consonance Gravity + Harmonic Pitch-Lock & Gate
//    - Exports genuine Xiph.org Ogg Vorbis I (.ogg) music discs & MC Resource Packs (.zip)
// ==============================================================================

const DM_SCALES = {
  phrygian: [0, 1, 4, 5, 7, 8, 10],    // Nether Phrygian Dominant
  dorian: [0, 2, 3, 5, 7, 9, 10],      // Aria Dorian Groove
  pentatonic: [0, 3, 5, 7, 10],        // Minor Pentatonic Funk
  lydian: [0, 2, 4, 6, 7, 9, 11],      // C418 Lydian Dream
  hirajoshi: [0, 2, 3, 7, 8],          // Cherry Blossom Hirajoshi
  mixolydian: [0, 2, 4, 5, 7, 9, 10]   // Upbeat Jam
};

// Classic Note Block Instruments (when placing a block under a Note Block)
const NOTEBLOCK_MELODIC_POOL = [
  { name: 'Note Pling', sound: 'note/pling.ogg', synth: 'square', timbre: 'pling', baseRate: 1.0, tex: 'block/glowstone.png', isNoteblock: true },
  { name: 'Chiptune Bit', sound: 'note/bit.ogg', synth: 'square', timbre: 'bit', baseRate: 1.0, tex: 'block/emerald_block.png', isNoteblock: true },
  { name: 'Bamboo Flute', sound: 'note/flute.ogg', synth: 'sine', timbre: 'flute', baseRate: 1.0, tex: 'item/bamboo.png', isNoteblock: true },
  { name: 'Iron Xylophone', sound: 'note/iron_xylophone.ogg', synth: 'triangle', timbre: 'xylophone', baseRate: 1.0, tex: 'item/iron_ingot.png', isNoteblock: true },
  { name: 'Bone Xylophone', sound: 'note/xylophone.ogg', synth: 'sine', timbre: 'xylophone', baseRate: 1.0, tex: 'item/bone.png', isNoteblock: true },
  { name: 'Hay Banjo', sound: 'note/banjo.ogg', synth: 'sawtooth', timbre: 'banjo', baseRate: 1.0, tex: 'item/wheat.png', isNoteblock: true }
];

const NOTEBLOCK_HARMONIC_POOL = [
  { name: 'Note Harp', sound: 'note/harp.ogg', synth: 'triangle', timbre: 'harp', baseRate: 1.0, tex: 'block/note_block.png', isNoteblock: true },
  { name: 'Wool Guitar', sound: 'note/guitar.ogg', synth: 'sawtooth', timbre: 'guitar', baseRate: 1.0, tex: 'block/white_wool.png', isNoteblock: true },
  { name: 'Ice Chime', sound: 'note/chime.ogg', synth: 'sine', timbre: 'chime', baseRate: 1.0, tex: 'item/amethyst_shard.png', isNoteblock: true },
  { name: 'Gold Bell', sound: 'note/bell.ogg', synth: 'sine', timbre: 'bell', baseRate: 1.0, tex: 'item/bell.png', isNoteblock: true }
];

const NOTEBLOCK_BASS_POOL = [
  { name: 'Wood Bass', sound: 'note/bass.ogg', synth: 'sawtooth', timbre: 'bass', baseRate: 1.0, tex: 'block/oak_log.png', isNoteblock: true },
  { name: 'Didgeridoo', sound: 'note/didgeridoo.ogg', synth: 'sawtooth', timbre: 'didgeridoo', baseRate: 1.0, tex: 'block/pumpkin_side.png', isNoteblock: true }
];

// Non-Noteblock Wild Minecraft Sounds (Mobs, Blocks, World, Items — Pitch-Quantized & ADSR-Gated!)
const WILD_MC_MELODIC_POOL = [
  { name: 'Amethyst Chime', sound: 'block/amethyst/chime1.ogg', synth: 'sine', timbre: 'amethyst', baseRate: 1.0, tex: 'item/amethyst_shard.png', isNoteblock: false },
  { name: 'Allay Song', sound: 'mob/allay/idle1.ogg', synth: 'sine', timbre: 'allay', baseRate: 1.05, tex: 'item/diamond.png', isNoteblock: false },
  { name: 'XP Orb Lead', sound: 'random/orb.ogg', synth: 'sine', timbre: 'orb', baseRate: 1.0, tex: 'item/experience_bottle.png', isNoteblock: false },
  { name: 'Enderman Warp', sound: 'mob/endermen/portal.ogg', synth: 'sawtooth', timbre: 'enderman', baseRate: 1.1, tex: 'item/ender_pearl.png', isNoteblock: false },
  { name: 'Cat Meow Synth', sound: 'mob/cat/meow1.ogg', synth: 'triangle', timbre: 'cat', baseRate: 1.0, tex: 'item/cod.png', isNoteblock: false },
  { name: 'Fox Squeak', sound: 'mob/fox/spit.ogg', synth: 'square', timbre: 'fox', baseRate: 1.1, tex: 'item/sweet_berries.png', isNoteblock: false },
  { name: 'Ghast Cry Lead', sound: 'mob/ghast/moan1.ogg', synth: 'sawtooth', timbre: 'ghast', baseRate: 0.95, tex: 'item/ghast_tear.png', isNoteblock: false },
  { name: 'LevelUp Fanfare', sound: 'random/levelup.ogg', synth: 'triangle', timbre: 'amethyst', baseRate: 1.0, tex: 'item/nether_star.png', isNoteblock: false },
  { name: 'Firework Twinkle', sound: 'fireworks/twinkle1.ogg', synth: 'sine', timbre: 'chime', baseRate: 1.15, tex: 'item/firework_rocket.png', isNoteblock: false }
];

const WILD_MC_HARMONIC_POOL = [
  { name: 'Villager Hmmm', sound: 'mob/villager/haggle1.ogg', synth: 'sawtooth', timbre: 'villager', baseRate: 1.0, tex: 'item/emerald.png', isNoteblock: false },
  { name: 'Portal Shimmer', sound: 'portal/portal.ogg', synth: 'sine', timbre: 'portal', baseRate: 1.0, tex: 'block/obsidian.png', isNoteblock: false },
  { name: 'Sculk Click', sound: 'block/sculk_sensor/sculk_clicking1.ogg', synth: 'triangle', timbre: 'sculk', baseRate: 1.1, tex: 'item/ender_eye.png', isNoteblock: false },
  { name: 'Glass Shatter', sound: 'random/glass1.ogg', synth: 'sine', timbre: 'glass', baseRate: 1.05, tex: 'block/glass.png', isNoteblock: false },
  { name: 'Bow String Pluck', sound: 'random/bow.ogg', synth: 'triangle', timbre: 'bow', baseRate: 1.0, tex: 'item/bow.png', isNoteblock: false },
  { name: 'Pig Oink Chord', sound: 'mob/pig/say1.ogg', synth: 'sawtooth', timbre: 'pig', baseRate: 1.0, tex: 'item/porkchop.png', isNoteblock: false },
  { name: 'Blaze Breathe', sound: 'mob/blaze/breathe1.ogg', synth: 'sawtooth', timbre: 'blaze', baseRate: 0.95, tex: 'item/blaze_rod.png', isNoteblock: false },
  { name: 'Totem Resonance', sound: 'item/totem/use_totem.ogg', synth: 'triangle', timbre: 'totem', baseRate: 1.0, tex: 'item/totem_of_undying.png', isNoteblock: false }
];

const WILD_MC_BASS_POOL = [
  { name: 'Anvil Drop Sub', sound: 'random/anvil_land.ogg', synth: 'square', timbre: 'anvil', baseRate: 0.72, tex: 'block/iron_block.png', isNoteblock: false },
  { name: 'Golem Hit Bass', sound: 'mob/irongolem/hit1.ogg', synth: 'sawtooth', timbre: 'golem', baseRate: 0.78, tex: 'item/iron_ingot.png', isNoteblock: false },
  { name: 'TNT Boom Sub', sound: 'random/explode1.ogg', synth: 'sawtooth', timbre: 'tnt', baseRate: 0.7, tex: 'block/tnt_side.png', isNoteblock: false },
  { name: 'Piston Thump', sound: 'tile/piston/out.ogg', synth: 'triangle', timbre: 'piston', baseRate: 0.82, tex: 'block/piston_side.png', isNoteblock: false },
  { name: 'Lava Pop Sub', sound: 'liquid/lavapop.ogg', synth: 'sine', timbre: 'lava', baseRate: 0.88, tex: 'item/lava_bucket.png', isNoteblock: false },
  { name: 'Slime Squish Bass', sound: 'mob/slime/big1.ogg', synth: 'triangle', timbre: 'slime', baseRate: 0.85, tex: 'item/slime_ball.png', isNoteblock: false },
  { name: 'Thunder Sub', sound: 'ambient/weather/thunder1.ogg', synth: 'sawtooth', timbre: 'tnt', baseRate: 0.72, tex: 'item/trident.png', isNoteblock: false }
];

const WILD_MC_FOLEY_KIT = [
  { fxType: 'orb', label: 'XP', name: 'XP Orb', sound: 'random/orb.ogg', tex: 'item/experience_bottle.png' },
  { fxType: 'pig', label: 'PIG', name: 'Pig Oink', sound: 'mob/pig/say1.ogg', tex: 'item/porkchop.png' },
  { fxType: 'anvil', label: 'ANV', name: 'Anvil Clang', sound: 'random/anvil_land.ogg', tex: 'block/iron_block.png' },
  { fxType: 'tnt', label: 'TNT', name: 'TNT Igniter', sound: 'random/fuse.ogg', tex: 'block/tnt_side.png' },
  { fxType: 'totem', label: 'TOT', name: 'Totem Pop', sound: 'item/totem/use_totem.ogg', tex: 'item/totem_of_undying.png' },
  { fxType: 'eat', label: 'EAT', name: 'Potato Crunch', sound: 'random/eat1.ogg', tex: 'item/baked_potato.png' },
  { fxType: 'ender', label: 'END', name: 'Ender Warp', sound: 'mob/endermen/portal.ogg', tex: 'item/ender_pearl.png' },
  { fxType: 'glass', label: 'GLS', name: 'Glass Clink', sound: 'random/glass1.ogg', tex: 'block/glass.png' }
];

const ALL_COMBINED_MELODIC_POOL = [...WILD_MC_MELODIC_POOL, ...NOTEBLOCK_MELODIC_POOL];
const ALL_COMBINED_HARMONIC_POOL = [...WILD_MC_HARMONIC_POOL, ...NOTEBLOCK_HARMONIC_POOL];
const ALL_COMBINED_BASS_POOL = [...WILD_MC_BASS_POOL, ...NOTEBLOCK_BASS_POOL];
const ALL_ANY_MC_VOICES = [...ALL_COMBINED_MELODIC_POOL, ...ALL_COMBINED_HARMONIC_POOL, ...ALL_COMBINED_BASS_POOL];

let dmTracks = [
  { id: 'lead', name: 'Amethyst Chime', tex: 'item/amethyst_shard.png', sound: 'block/amethyst/chime1.ogg', synthType: 'sine', timbre: 'amethyst', baseRate: 1.0, css: 'track-lead' },
  { id: 'arp', name: 'Villager Hmmm', tex: 'item/emerald.png', sound: 'mob/villager/haggle1.ogg', synthType: 'sawtooth', timbre: 'villager', baseRate: 1.0, css: 'track-arp' },
  { id: 'bell', name: 'Enderman Warp', tex: 'item/ender_pearl.png', sound: 'mob/endermen/portal.ogg', synthType: 'sawtooth', timbre: 'enderman', baseRate: 1.1, css: 'track-bell' },
  { id: 'bass', name: 'Anvil Drop Sub', tex: 'block/iron_block.png', sound: 'random/anvil_land.ogg', synthType: 'square', timbre: 'anvil', baseRate: 0.72, css: 'track-bass' },
  { id: 'drums', name: 'MC World Kit', tex: 'block/tnt_side.png', sound: 'note/bd.ogg', synthType: 'drum', timbre: 'drum', baseRate: 1.0, css: 'track-drums' },
  { id: 'fx', name: 'Any MC Foley', tex: 'item/experience_bottle.png', sound: 'random/orb.ogg', synthType: 'fx', timbre: 'fx', baseRate: 1.0, css: 'track-fx' }
];

let dmPlaying = false;
let dmAutoMutate = true;
let dmStep = 0;
let dmBar = 0;
let dmTimerId = null;
let dmCurrentChords = [0, 5, 2, 6];
let dmPattern = Array.from({ length: 6 }, () => Array(16).fill(null));
let lastExportedOggBlobUrl = null;

/**
 * Picks a dynamic random sound from the 1,000+ official Minecraft Asset Index entries in SOUND_SHA1_CACHE
 * when available, or from the curated wild Minecraft sound pools.
 */
function pickRandomMinecraftVoiceFromPool(role = 'melodic', poolMode = 'all_mc_random') {
  if (poolMode === 'noteblock_only') {
    if (role === 'bass') return NOTEBLOCK_BASS_POOL[Math.floor(Math.random() * NOTEBLOCK_BASS_POOL.length)];
    if (role === 'harmonic') return NOTEBLOCK_HARMONIC_POOL[Math.floor(Math.random() * NOTEBLOCK_HARMONIC_POOL.length)];
    return NOTEBLOCK_MELODIC_POOL[Math.floor(Math.random() * NOTEBLOCK_MELODIC_POOL.length)];
  }

  // Try occasionally sampling a brand-new sound path from SOUND_SHA1_CACHE (the full Minecraft asset index)
  const shaKeys = Object.keys(SOUND_SHA1_CACHE);
  if (shaKeys.length > 25 && Math.random() < 0.28) {
    const filtered = shaKeys.filter((k) => !k.includes('records/') && !k.includes('music/') && !k.includes('ambient/cave'));
    if (filtered.length > 0) {
      const chosenKey = filtered[Math.floor(Math.random() * filtered.length)];
      const relPath = chosenKey.replace(/^minecraft\/sounds\//, '');
      const parts = relPath.replace('.ogg', '').split('/');
      const shortLabel = parts.slice(-2).join('/').slice(0, 15);
      // Asynchronously warm the buffer cache for this asset-index sound
      ensureSoundDecodedInCache(relPath);
      const fallbackTimbres = ['amethyst', 'villager', 'enderman', 'cat', 'anvil', 'piston', 'slime', 'glass'];
      const fallbackTextures = [
        'item/amethyst_shard.png', 'item/ender_pearl.png', 'item/emerald.png',
        'block/tnt_side.png', 'item/blaze_rod.png', 'item/slime_ball.png', 'item/totem_of_undying.png'
      ];
      return {
        name: `MC: ${shortLabel}`,
        sound: relPath,
        synth: role === 'bass' ? 'sawtooth' : 'triangle',
        timbre: fallbackTimbres[Math.floor(Math.random() * fallbackTimbres.length)],
        baseRate: role === 'bass' ? 0.78 : 1.0,
        tex: fallbackTextures[Math.floor(Math.random() * fallbackTextures.length)],
        isNoteblock: false
      };
    }
  }

  if (poolMode === 'pure_non_noteblock' || poolMode === 'per_note_chaos') {
    if (role === 'bass') return WILD_MC_BASS_POOL[Math.floor(Math.random() * WILD_MC_BASS_POOL.length)];
    if (role === 'harmonic') return WILD_MC_HARMONIC_POOL[Math.floor(Math.random() * WILD_MC_HARMONIC_POOL.length)];
    return WILD_MC_MELODIC_POOL[Math.floor(Math.random() * WILD_MC_MELODIC_POOL.length)];
  }

  // Default 'all_mc_random': combines all wild Minecraft game sounds + Note Block instruments
  if (role === 'bass') return ALL_COMBINED_BASS_POOL[Math.floor(Math.random() * ALL_COMBINED_BASS_POOL.length)];
  if (role === 'harmonic') return ALL_COMBINED_HARMONIC_POOL[Math.floor(Math.random() * ALL_COMBINED_HARMONIC_POOL.length)];
  return ALL_COMBINED_MELODIC_POOL[Math.floor(Math.random() * ALL_COMBINED_MELODIC_POOL.length)];
}

function ensureSoundDecodedInCache(relPath) {
  if (!relPath || DECODED_BUFFER_CACHE[relPath]) return;
  const urls = getSoundUrls(relPath);
  (async () => {
    for (const u of urls) {
      try {
        const res = await fetch(u);
        if (!res.ok) continue;
        const arr = await res.arrayBuffer();
        const ctx = ensureAudioCtx();
        if (ctx) {
          const buf = await ctx.decodeAudioData(arr);
          DECODED_BUFFER_CACHE[relPath] = buf;
          break;
        }
      } catch (_) {}
    }
  })();
}

/**
 * Bjorklund's Euclidean Rhythm Generator E(pulses, steps, rotation)
 */
function generateEuclideanRhythm(pulses, steps, rotation = 0) {
  const k = Math.max(0, Math.min(steps, Math.round(pulses)));
  const pattern = new Array(steps).fill(false);
  if (k === 0) return pattern;
  for (let i = 0; i < steps; i++) {
    if (((i * k) % steps) < k) {
      pattern[(i + rotation) % steps] = true;
    }
  }
  return pattern;
}

/**
 * Markov + Consonance-Gravity Pitch Picker
 */
function pickStochasticScaleDegree(prevDegree, isStrongBeat, consonanceWeight) {
  const candidates = [0, 1, 2, 3, 4, 5, 6];
  const weights = candidates.map((deg) => {
    const isChordTone = deg === 0 || deg === 2 || deg === 4;
    const dist = Math.abs(deg - prevDegree);
    const proximityScore = 1 / (1 + dist * 0.75);
    const harmonicScore = isChordTone
      ? (isStrongBeat ? consonanceWeight * 4.5 : consonanceWeight * 2.2)
      : (1 - consonanceWeight * 0.7) * 1.8;
    return proximityScore * harmonicScore;
  });

  const totalWeight = weights.reduce((a, b) => a + b, 0);
  let r = Math.random() * totalWeight;
  for (let i = 0; i < candidates.length; i++) {
    r -= weights[i];
    if (r <= 0) return candidates[i];
  }
  return 0;
}

function initDiscMakerStudio() {
  setInterval(() => {
    const randomDiscTex = ALL_MINECRAFT_DISCS[Math.floor(Math.random() * ALL_MINECRAFT_DISCS.length)];
    document.querySelectorAll('.disc-maker-cycling-icon').forEach((img) => {
      attachMultiSourceImage(img, getTextureUrls(randomDiscTex));
    });
    const targetDiscSel = document.getElementById('dm-target-disc-select');
    const chosenDisc = targetDiscSel ? targetDiscSel.value : randomDiscTex.replace('item/music_disc_', '').replace('.png', '');
    const poolSelect = document.getElementById('dm-sound-pool-select');
    const poolLabel = poolSelect ? poolSelect.options[poolSelect.selectedIndex]?.text.split('(')[0].trim() : 'Total Random';
    const readout = document.getElementById('dm-active-disc-label');
    if (readout) {
      readout.textContent = `Target Disc: ${chosenDisc}.ogg • Mode: ${poolLabel}`;
    }
  }, 1500);

  preloadNoteBlockSamples();
  buildSequencerDOM();
  generateStochasticMusicalPattern(true);

  const playBtn = document.getElementById('dm-play-btn');
  const turntableBtn = document.getElementById('dm-turntable-btn');
  const randBtn = document.getElementById('dm-randomize-btn');
  const mutateBtn = document.getElementById('dm-mutate-btn');
  const bpmSlider = document.getElementById('dm-bpm-slider');
  const swingSlider = document.getElementById('dm-swing-slider');
  const densitySlider = document.getElementById('dm-density-slider');
  const consonanceSlider = document.getElementById('dm-consonance-slider');
  const harmonicLockSlider = document.getElementById('dm-harmonic-lock-slider');
  const scaleSelect = document.getElementById('dm-scale-select');
  const rootSelect = document.getElementById('dm-root-select');
  const poolSelect = document.getElementById('dm-sound-pool-select');
  const targetDiscSelect = document.getElementById('dm-target-disc-select');
  const exportOggBtn = document.getElementById('dm-export-ogg-btn');
  const exportPackBtn = document.getElementById('dm-export-pack-btn');

  const toggleStudioPlay = () => {
    if (dmPlaying) stopDiscMaker();
    else startDiscMaker();
  };

  if (playBtn) playBtn.addEventListener('click', toggleStudioPlay);
  if (turntableBtn) turntableBtn.addEventListener('click', toggleStudioPlay);

  if (randBtn) {
    randBtn.addEventListener('click', () => {
      randomizeAllStudioParameters();
      generateStochasticMusicalPattern(true);
      if (!dmPlaying) startDiscMaker();
      showAdvancementToast('disc_maker Randomized!', `${dmTracks[0].name} + ${dmTracks[1].name} + ${dmTracks[3].name}`, 'item/music_disc_creator.png');
    });
  }

  if (mutateBtn) {
    mutateBtn.addEventListener('click', () => {
      dmAutoMutate = !dmAutoMutate;
      const lbl = document.getElementById('dm-mutate-label');
      if (lbl) lbl.textContent = `Live Mutate: ${dmAutoMutate ? 'ON' : 'OFF'}`;
      playMcClick(1.1);
    });
  }

  if (bpmSlider) {
    bpmSlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-bpm-val');
      if (el) el.textContent = `${e.target.value} BPM`;
    });
  }
  if (swingSlider) {
    swingSlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-swing-val');
      if (el) el.textContent = `${e.target.value}%`;
    });
  }
  if (consonanceSlider) {
    consonanceSlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-consonance-val');
      if (el) el.textContent = `${e.target.value}%`;
      generateStochasticMusicalPattern(false);
    });
  }
  if (harmonicLockSlider) {
    harmonicLockSlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-harmonic-lock-val');
      if (el) el.textContent = `${e.target.value}%`;
    });
  }
  if (densitySlider) {
    densitySlider.addEventListener('input', (e) => {
      const el = document.getElementById('dm-density-val');
      if (el) el.textContent = `${e.target.value}%`;
      generateStochasticMusicalPattern(false);
    });
  }
  if (scaleSelect) scaleSelect.addEventListener('change', () => generateStochasticMusicalPattern(false));
  if (rootSelect) rootSelect.addEventListener('change', () => generateStochasticMusicalPattern(false));
  if (poolSelect) {
    poolSelect.addEventListener('change', () => {
      generateStochasticMusicalPattern(true);
      showAdvancementToast('Sound Palette Updated', poolSelect.options[poolSelect.selectedIndex].text, 'block/note_block.png');
    });
  }

  if (targetDiscSelect) {
    targetDiscSelect.addEventListener('change', () => {
      const disc = targetDiscSelect.value || 'pigstep';
      const lbl = document.getElementById('dm-export-ogg-label');
      if (lbl) lbl.textContent = `EXPORT .OGG DISC (${disc}.ogg)`;
    });
  }

  if (exportOggBtn) {
    exportOggBtn.addEventListener('click', () => {
      exportDiscMakerAsOggFile(false);
    });
  }

  if (exportPackBtn) {
    exportPackBtn.addEventListener('click', () => {
      exportDiscMakerAsOggFile(true);
    });
  }

  window.startDiscMakerFromConsole = () => {
    randomizeAllStudioParameters();
    generateStochasticMusicalPattern(true);
    startDiscMaker();
  };
}

function buildSequencerDOM() {
  const gridContainer = document.getElementById('dm-sequencer-grid');
  if (!gridContainer) return;
  gridContainer.innerHTML = '';

  dmTracks.forEach((track, tIdx) => {
    const row = document.createElement('div');
    row.className = 'sequencer-row';

    const label = document.createElement('div');
    label.className = 'seq-track-label';
    label.style.cursor = 'pointer';
    label.title = 'Click to roll a new random Minecraft sound for this track';
    label.addEventListener('click', () => {
      if (tIdx < 4) {
        const poolMode = document.getElementById('dm-sound-pool-select')?.value || 'all_mc_random';
        const role = tIdx === 3 ? 'bass' : (tIdx === 0 ? 'melodic' : 'harmonic');
        const v = pickRandomMinecraftVoiceFromPool(role, poolMode);
        dmTracks[tIdx] = { ...dmTracks[tIdx], name: v.name, sound: v.sound, synthType: v.synth, timbre: v.timbre, baseRate: v.baseRate || 1.0, tex: v.tex };
        const iconEl = document.getElementById(`seq-track-icon-${tIdx}`);
        const nameEl = document.getElementById(`seq-track-name-${tIdx}`);
        if (iconEl) attachMultiSourceImage(iconEl, getTextureUrls(v.tex));
        if (nameEl) nameEl.textContent = v.name;
        playQuantizedMinecraftNote(dmTracks[tIdx], 0);
      }
    });

    const icon = document.createElement('img');
    icon.className = 'mc-png mc-png-sm';
    icon.id = `seq-track-icon-${tIdx}`;
    attachMultiSourceImage(icon, getTextureUrls(track.tex));
    const text = document.createElement('span');
    text.className = 'mc-f';
    text.id = `seq-track-name-${tIdx}`;
    text.textContent = track.name;
    label.appendChild(icon);
    label.appendChild(text);
    row.appendChild(label);

    for (let s = 0; s < 16; s++) {
      const cell = document.createElement('div');
      cell.className = `seq-cell ${s % 4 === 0 ? 'beat-marker' : ''} ${track.css}`;
      cell.dataset.track = String(tIdx);
      cell.dataset.step = String(s);

      cell.addEventListener('click', () => {
        toggleSequencerCell(tIdx, s);
        playMcClick(1.2);
      });

      row.appendChild(cell);
    }
    gridContainer.appendChild(row);
  });
}

function randomizeAllStudioParameters() {
  const scaleSelect = document.getElementById('dm-scale-select');
  const rootSelect = document.getElementById('dm-root-select');
  const bpmSlider = document.getElementById('dm-bpm-slider');
  const swingSlider = document.getElementById('dm-swing-slider');
  const densitySlider = document.getElementById('dm-density-slider');
  const consonanceSlider = document.getElementById('dm-consonance-slider');
  const harmonicLockSlider = document.getElementById('dm-harmonic-lock-slider');

  if (scaleSelect) scaleSelect.selectedIndex = Math.floor(Math.random() * scaleSelect.options.length);
  if (rootSelect) rootSelect.selectedIndex = Math.floor(Math.random() * rootSelect.options.length);

  const randomBpm = Math.floor(96 + Math.random() * 42);
  const randomSwing = Math.floor(8 + Math.random() * 24);
  const randomDensity = Math.floor(48 + Math.random() * 36);
  const randomConsonance = Math.floor(74 + Math.random() * 22);
  const randomHarmonicLock = Math.floor(78 + Math.random() * 20);

  if (bpmSlider) {
    bpmSlider.value = String(randomBpm);
    const el = document.getElementById('dm-bpm-val');
    if (el) el.textContent = `${randomBpm} BPM`;
  }
  if (swingSlider) {
    swingSlider.value = String(randomSwing);
    const el = document.getElementById('dm-swing-val');
    if (el) el.textContent = `${randomSwing}%`;
  }
  if (densitySlider) {
    densitySlider.value = String(randomDensity);
    const el = document.getElementById('dm-density-val');
    if (el) el.textContent = `${randomDensity}%`;
  }
  if (consonanceSlider) {
    consonanceSlider.value = String(randomConsonance);
    const el = document.getElementById('dm-consonance-val');
    if (el) el.textContent = `${randomConsonance}%`;
  }
  if (harmonicLockSlider) {
    harmonicLockSlider.value = String(randomHarmonicLock);
    const el = document.getElementById('dm-harmonic-lock-val');
    if (el) el.textContent = `${randomHarmonicLock}%`;
  }
}

/**
 * 100% Stochastic Pattern Generator supporting Any Minecraft Sound!
 */
function generateStochasticMusicalPattern(randomizeVoices = false) {
  const poolMode = document.getElementById('dm-sound-pool-select')?.value || 'all_mc_random';

  if (randomizeVoices) {
    const leadVoice = pickRandomMinecraftVoiceFromPool('melodic', poolMode);
    const arpVoice = pickRandomMinecraftVoiceFromPool('harmonic', poolMode);
    const bellVoice = pickRandomMinecraftVoiceFromPool('melodic', poolMode);
    const bassVoice = pickRandomMinecraftVoiceFromPool('bass', poolMode);

    dmTracks[0] = { ...dmTracks[0], name: leadVoice.name, sound: leadVoice.sound, synthType: leadVoice.synth, timbre: leadVoice.timbre, baseRate: leadVoice.baseRate || 1.0, tex: leadVoice.tex };
    dmTracks[1] = { ...dmTracks[1], name: arpVoice.name, sound: arpVoice.sound, synthType: arpVoice.synth, timbre: arpVoice.timbre, baseRate: arpVoice.baseRate || 1.0, tex: arpVoice.tex };
    dmTracks[2] = { ...dmTracks[2], name: bellVoice.name, sound: bellVoice.sound, synthType: bellVoice.synth, timbre: bellVoice.timbre, baseRate: bellVoice.baseRate || 1.0, tex: bellVoice.tex };
    dmTracks[3] = { ...dmTracks[3], name: bassVoice.name, sound: bassVoice.sound, synthType: bassVoice.synth, timbre: bassVoice.timbre, baseRate: bassVoice.baseRate || 1.0, tex: bassVoice.tex };

    for (let i = 0; i < 4; i++) {
      const iconEl = document.getElementById(`seq-track-icon-${i}`);
      const nameEl = document.getElementById(`seq-track-name-${i}`);
      if (iconEl) attachMultiSourceImage(iconEl, getTextureUrls(dmTracks[i].tex));
      if (nameEl) nameEl.textContent = dmTracks[i].name;
    }
  }

  const chordPool = [1, 2, 3, 4, 5, 6];
  dmCurrentChords = [
    0,
    chordPool[Math.floor(Math.random() * chordPool.length)],
    chordPool[Math.floor(Math.random() * chordPool.length)],
    chordPool[Math.floor(Math.random() * chordPool.length)]
  ];

  const density = (parseInt(document.getElementById('dm-density-slider')?.value || '64', 10) || 64) / 100;
  const consonance = (parseInt(document.getElementById('dm-consonance-slider')?.value || '84', 10) || 84) / 100;

  const leadPulses = Math.max(4, Math.min(12, Math.round(density * 11 + (Math.random() * 3 - 1.5))));
  const arpPulses = Math.max(5, Math.min(13, Math.round(density * 12 + (Math.random() * 3 - 1.5))));
  const bellPulses = Math.max(2, Math.min(7, Math.round(density * 5 + Math.random() * 2)));
  const bassPulses = Math.max(4, Math.min(9, Math.round(density * 8 + Math.random() * 2)));
  const kickPulses = Math.max(3, Math.min(6, Math.round(3 + density * 3)));
  const hatPulses = Math.max(5, Math.min(14, Math.round(density * 13)));
  const foleyPulses = Math.max(2, Math.min(6, Math.round(2 + density * 4)));

  const leadEuc = generateEuclideanRhythm(leadPulses, 16, Math.floor(Math.random() * 4));
  const arpEuc = generateEuclideanRhythm(arpPulses, 16, Math.floor(Math.random() * 4));
  const bellEuc = generateEuclideanRhythm(bellPulses, 16, Math.floor(Math.random() * 8));
  const bassEuc = generateEuclideanRhythm(bassPulses, 16, 0);
  const kickEuc = generateEuclideanRhythm(kickPulses, 16, 0);
  const hatEuc = generateEuclideanRhythm(hatPulses, 16, Math.floor(Math.random() * 2));
  const foleyEuc = generateEuclideanRhythm(foleyPulses, 16, Math.floor(Math.random() * 5));

  const newPat = Array.from({ length: 6 }, () => Array(16).fill(null));
  let prevLeadDeg = 0;
  let prevArpDeg = 2;

  for (let s = 0; s < 16; s++) {
    const isStrongBeat = s % 4 === 0;
    const isBackbeat = s % 8 === 4;

    if (leadEuc[s]) {
      const deg = pickStochasticScaleDegree(prevLeadDeg, isStrongBeat, consonance);
      prevLeadDeg = deg;
      const oct = Math.random() < 0.22 ? 1 : 0;
      const stepVoice = poolMode === 'per_note_chaos' ? pickRandomMinecraftVoiceFromPool('melodic', 'all_mc_random') : null;
      newPat[0][s] = { degree: deg, octave: oct, label: stepVoice ? stepVoice.name.slice(0, 2).toUpperCase() : String(deg + 1), customVoice: stepVoice };
    }

    if (arpEuc[s]) {
      const deg = pickStochasticScaleDegree(prevArpDeg, true, Math.min(0.98, consonance + 0.1));
      prevArpDeg = deg;
      const stepVoice = poolMode === 'per_note_chaos' ? pickRandomMinecraftVoiceFromPool('harmonic', 'all_mc_random') : null;
      newPat[1][s] = { degree: deg, octave: 0, label: stepVoice ? stepVoice.name.slice(0, 2).toUpperCase() : String(deg + 1), customVoice: stepVoice };
    }

    if (bellEuc[s]) {
      const deg = pickStochasticScaleDegree(4, true, 0.92);
      const stepVoice = poolMode === 'per_note_chaos' ? pickRandomMinecraftVoiceFromPool('melodic', 'all_mc_random') : null;
      newPat[2][s] = { degree: deg, octave: 1, label: stepVoice ? stepVoice.name.slice(0, 2).toUpperCase() : '★', customVoice: stepVoice };
    }

    if (bassEuc[s]) {
      const deg = isStrongBeat || Math.random() < consonance ? 0 : 4;
      const stepVoice = poolMode === 'per_note_chaos' ? pickRandomMinecraftVoiceFromPool('bass', 'all_mc_random') : null;
      newPat[3][s] = { degree: deg, octave: -1, label: deg === 0 ? 'R' : '5', customVoice: stepVoice };
    }

    if (isBackbeat) {
      newPat[4][s] = { drumType: 'snare', label: 'S' };
    } else if (kickEuc[s]) {
      newPat[4][s] = { drumType: 'kick', label: 'K' };
    } else if (hatEuc[s]) {
      newPat[4][s] = { drumType: 'hat', label: 'H' };
    }

    if (foleyEuc[s] && (poolMode !== 'noteblock_only' || isStrongBeat)) {
      const foleyItem = WILD_MC_FOLEY_KIT[Math.floor(Math.random() * WILD_MC_FOLEY_KIT.length)];
      const deg = pickStochasticScaleDegree(0, true, 0.9);
      newPat[5][s] = { fxType: foleyItem.fxType, sound: foleyItem.sound, degree: deg, label: foleyItem.label };
    }
  }

  dmPattern = newPat;
  renderSequencerGridUI();
}

function toggleSequencerCell(tIdx, sIdx) {
  if (dmPattern[tIdx][sIdx]) {
    dmPattern[tIdx][sIdx] = null;
  } else {
    if (tIdx === 4) {
      dmPattern[tIdx][sIdx] = { drumType: sIdx % 4 === 0 ? 'kick' : 'snare', label: sIdx % 4 === 0 ? 'K' : 'S' };
    } else if (tIdx === 5) {
      const foleyItem = WILD_MC_FOLEY_KIT[Math.floor(Math.random() * WILD_MC_FOLEY_KIT.length)];
      dmPattern[tIdx][sIdx] = { fxType: foleyItem.fxType, sound: foleyItem.sound, degree: 0, label: foleyItem.label };
    } else {
      const deg = pickStochasticScaleDegree(0, sIdx % 4 === 0, 0.85);
      dmPattern[tIdx][sIdx] = { degree: deg, octave: tIdx === 3 ? -1 : 0, label: String(deg + 1) };
    }
  }
  renderSequencerGridUI();
}

function renderSequencerGridUI() {
  document.querySelectorAll('#dm-sequencer-grid .seq-cell').forEach((cell) => {
    const t = parseInt(cell.dataset.track, 10);
    const s = parseInt(cell.dataset.step, 10);
    const data = dmPattern[t]?.[s];
    cell.classList.toggle('active-step', Boolean(data));
    cell.textContent = data ? (data.label || '●') : '';
  });
}

function startDiscMaker() {
  if (bgMusicAudio && bgMusicPlaying) {
    bgMusicAudio.pause();
    bgMusicPlaying = false;
    updateJukeboxUI();
  }

  ensureAudioCtx();
  dmPlaying = true;
  dmStep = 0;
  dmBar = 0;

  const playLabel = document.getElementById('dm-play-label');
  const badge = document.getElementById('dm-status-badge');
  const mainDisc = document.getElementById('dm-main-disc-icon');
  if (playLabel) playLabel.textContent = 'STOP COMPOSER';
  if (badge) badge.textContent = 'COMPOSING LIVE';
  if (mainDisc) mainDisc.classList.add('disc-spinning');

  scheduleNextDiscMakerStep();
}

function stopDiscMaker() {
  dmPlaying = false;
  if (dmTimerId) {
    clearTimeout(dmTimerId);
    dmTimerId = null;
  }
  const playLabel = document.getElementById('dm-play-label');
  const badge = document.getElementById('dm-status-badge');
  const mainDisc = document.getElementById('dm-main-disc-icon');
  if (playLabel) playLabel.textContent = 'START COMPOSER';
  if (badge) badge.textContent = 'PAUSED';
  if (mainDisc) mainDisc.classList.remove('disc-spinning');

  document.querySelectorAll('#dm-sequencer-grid .seq-cell.playhead-now').forEach((c) => {
    c.classList.remove('playhead-now');
  });
}

function scheduleNextDiscMakerStep() {
  if (!dmPlaying) return;

  const bpm = parseInt(document.getElementById('dm-bpm-slider')?.value || '114', 10) || 114;
  const swingPct = (parseInt(document.getElementById('dm-swing-slider')?.value || '20', 10) || 20) / 100;

  const sixteenthMs = (60000 / bpm) / 4;
  const currentStepDuration = (dmStep % 2 === 0)
    ? sixteenthMs * (1 + swingPct * 0.5)
    : sixteenthMs * (1 - swingPct * 0.5);

  playDiscMakerStep(dmStep, dmBar);

  dmStep++;
  if (dmStep >= 16) {
    dmStep = 0;
    dmBar = (dmBar + 1) % 4;
    if (dmBar === 0 && dmAutoMutate) {
      generateStochasticMusicalPattern(Math.random() < 0.45);
    }
  }

  dmTimerId = setTimeout(scheduleNextDiscMakerStep, currentStepDuration);
}

function playDiscMakerStep(stepIdx, barIdx) {
  document.querySelectorAll('#dm-sequencer-grid .seq-cell').forEach((cell) => {
    const s = parseInt(cell.dataset.step, 10);
    cell.classList.toggle('playhead-now', s === stepIdx);
  });

  const scaleKey = document.getElementById('dm-scale-select')?.value || 'phrygian';
  const scale = DM_SCALES[scaleKey] || DM_SCALES.phrygian;
  const rootOffset = parseInt(document.getElementById('dm-root-select')?.value || '0', 10) || 0;
  const activeChordDegree = dmCurrentChords[barIdx % dmCurrentChords.length] || 0;

  const readout = document.getElementById('dm-chord-readout');
  if (readout) {
    readout.textContent = `Bar ${barIdx + 1}/4 • Chord Root: +${activeChordDegree} • Step ${stepIdx + 1}/16 • Pitch-Locked`;
  }

  for (let t = 0; t < dmTracks.length; t++) {
    const cellData = dmPattern[t][stepIdx];
    if (!cellData) continue;
    const baseTrack = dmTracks[t];
    const activeVoice = cellData.customVoice ? { ...baseTrack, ...cellData.customVoice, synthType: cellData.customVoice.synth } : baseTrack;

    if (baseTrack.id === 'drums') {
      playComposerDrum(cellData.drumType);
    } else if (baseTrack.id === 'fx') {
      const foleySemi = rootOffset + (scale[(activeChordDegree + (cellData.degree || 0)) % scale.length] || 0);
      playComposerFoley(cellData.fxType, foleySemi, cellData.sound);
    } else {
      const totalDegree = activeChordDegree + (cellData.degree || 0);
      const scaleLen = scale.length;
      const octaveFromDegree = Math.floor(totalDegree / scaleLen);
      const scaleSemitone = scale[((totalDegree % scaleLen) + scaleLen) % scaleLen];
      const semitoneOffset = rootOffset + scaleSemitone + (octaveFromDegree + (cellData.octave || 0)) * 12;

      playQuantizedMinecraftNote(activeVoice, semitoneOffset);
    }
  }
}

function preloadNoteBlockSamples() {
  const paths = [
    'note/pling.ogg', 'note/harp.ogg', 'note/bell.ogg', 'note/bass.ogg',
    'note/bit.ogg', 'note/flute.ogg', 'note/guitar.ogg', 'note/chime.ogg',
    'note/xylophone.ogg', 'note/iron_xylophone.ogg', 'note/banjo.ogg', 'note/didgeridoo.ogg',
    'note/bd.ogg', 'note/snare.ogg', 'note/hat.ogg',
    'random/orb.ogg', 'mob/pig/say1.ogg', 'random/anvil_land.ogg',
    'mob/endermen/portal.ogg', 'mob/cat/meow1.ogg', 'mob/villager/haggle1.ogg',
    'random/glass1.ogg', 'random/bow.ogg', 'liquid/lavapop.ogg',
    'mob/slime/big1.ogg', 'random/explode1.ogg', 'item/totem/use_totem.ogg',
    'random/eat1.ogg', 'random/levelup.ogg', 'fireworks/twinkle1.ogg'
  ];

  paths.forEach((p) => ensureSoundDecodedInCache(p));
}

function playQuantizedMinecraftNote(track, semitoneOffset) {
  if (!sfxEnabled) return;
  const ctx = ensureAudioCtx();
  if (!ctx) return;

  const harmonicLock = (parseInt(document.getElementById('dm-harmonic-lock-slider')?.value || '88', 10) || 88) / 100;
  const baseRate = track.baseRate || 1.0;
  const exactRate = baseRate * Math.pow(2, semitoneOffset / 12);
  // When Harmonic Pitch-Lock < 100%, allow subtle organic Minecraft sample pitch character
  const lockedRate = exactRate * harmonicLock + baseRate * (1 - harmonicLock);
  const clampedRate = Math.max(0.35, Math.min(2.8, lockedRate));
  const now = ctx.currentTime;
  const gateDecay = track.id === 'bass' ? 0.28 : (track.id === 'bell' ? 0.36 : 0.24);

  const buf = DECODED_BUFFER_CACHE[track.sound];
  if (buf) {
    const src = ctx.createBufferSource();
    const gain = ctx.createGain();
    const filter = ctx.createBiquadFilter();
    src.buffer = buf;
    src.playbackRate.setValueAtTime(clampedRate, now);

    // Apply musical ADSR gate and resonant filter so even long non-noteblock Minecraft sounds lock tightly to the groove!
    filter.type = track.id === 'bass' ? 'lowpass' : 'bandpass';
    const targetFreq = (track.id === 'bass' ? 110 : 370) * Math.pow(2, semitoneOffset / 12);
    filter.frequency.setValueAtTime(Math.max(80, Math.min(4800, targetFreq * 2.2)), now);
    filter.Q.setValueAtTime(track.isNoteblock ? 0.7 : (1.2 + harmonicLock * 2.5), now);

    const peak = track.id === 'bass' ? 0.38 : 0.26;
    gain.gain.setValueAtTime(peak, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + gateDecay + 0.08);

    src.connect(filter);
    filter.connect(gain);
    gain.connect(ctx.destination);
    src.start(now);
    src.stop(now + gateDecay + 0.1);
  }

  // Layer pitch-locked Minecraft character formant/harmonic voice so every sound is 100% in key!
  const osc = ctx.createOscillator();
  const subOsc = ctx.createOscillator();
  const gain = ctx.createGain();
  const filter = ctx.createBiquadFilter();

  const baseHz = track.id === 'bass' ? 92.5 : (track.id === 'bell' ? 739.99 : 369.99);
  const freq = baseHz * Math.pow(2, semitoneOffset / 12);
  const timbre = track.timbre || 'pling';

  osc.type = track.synthType || 'triangle';
  subOsc.type = timbre === 'anvil' || timbre === 'amethyst' ? 'square' : 'sine';

  if (timbre === 'enderman' || timbre === 'portal') {
    osc.frequency.setValueAtTime(freq * 1.06, now);
    osc.frequency.exponentialRampToValueAtTime(freq, now + 0.07);
    subOsc.frequency.setValueAtTime(freq * 1.5, now);
  } else if (timbre === 'villager' || timbre === 'cat' || timbre === 'pig' || timbre === 'ghast') {
    osc.frequency.setValueAtTime(freq * 0.94, now);
    osc.frequency.linearRampToValueAtTime(freq * 1.02, now + 0.05);
    osc.frequency.linearRampToValueAtTime(freq, now + 0.14);
    subOsc.frequency.setValueAtTime(freq * 2.0, now);
  } else if (timbre === 'anvil' || timbre === 'glass') {
    osc.frequency.setValueAtTime(freq, now);
    subOsc.frequency.setValueAtTime(freq * 2.756, now); // Metallic inharmonic overtone locked to root
  } else if (timbre === 'slime' || timbre === 'lava' || timbre === 'piston') {
    osc.frequency.setValueAtTime(freq * 1.35, now);
    osc.frequency.exponentialRampToValueAtTime(freq, now + 0.08);
    subOsc.frequency.setValueAtTime(freq * 0.5, now);
  } else {
    osc.frequency.setValueAtTime(freq, now);
    subOsc.frequency.setValueAtTime(freq * 2.0, now);
  }

  filter.type = 'lowpass';
  filter.frequency.setValueAtTime(track.id === 'bass' ? 480 : 2800, now);

  const synthScale = buf ? (0.45 * harmonicLock) : 1.0;
  const peakGain = (track.id === 'bass' ? 0.15 : 0.095) * synthScale;

  gain.gain.setValueAtTime(Math.max(0.002, peakGain), now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + gateDecay);

  osc.connect(filter);
  subOsc.connect(filter);
  filter.connect(gain);
  gain.connect(ctx.destination);

  osc.start(now);
  subOsc.start(now);
  osc.stop(now + gateDecay + 0.02);
  subOsc.stop(now + gateDecay + 0.02);
}

function playComposerDrum(drumType) {
  if (!sfxEnabled) return;
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;

  if (drumType === 'kick') {
    const buf = DECODED_BUFFER_CACHE['note/bd.ogg'];
    if (buf) {
      const src = ctx.createBufferSource();
      const g = ctx.createGain();
      src.buffer = buf;
      g.gain.setValueAtTime(0.5, now);
      src.connect(g);
      g.connect(ctx.destination);
      src.start(now);
    }
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'sine';
    osc.frequency.setValueAtTime(135, now);
    osc.frequency.exponentialRampToValueAtTime(38, now + 0.12);
    gain.gain.setValueAtTime(0.22, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.14);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.15);
  } else if (drumType === 'snare') {
    const buf = DECODED_BUFFER_CACHE['note/snare.ogg'];
    if (buf) {
      const src = ctx.createBufferSource();
      const g = ctx.createGain();
      src.buffer = buf;
      g.gain.setValueAtTime(0.42, now);
      src.connect(g);
      g.connect(ctx.destination);
      src.start(now);
      return;
    }
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'triangle';
    osc.frequency.setValueAtTime(220, now);
    osc.frequency.exponentialRampToValueAtTime(95, now + 0.09);
    gain.gain.setValueAtTime(0.14, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.11);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.12);
  } else {
    const buf = DECODED_BUFFER_CACHE['note/hat.ogg'];
    if (buf) {
      const src = ctx.createBufferSource();
      const g = ctx.createGain();
      src.buffer = buf;
      g.gain.setValueAtTime(0.26, now);
      src.connect(g);
      g.connect(ctx.destination);
      src.start(now);
      return;
    }
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'square';
    osc.frequency.setValueAtTime(3200, now);
    gain.gain.setValueAtTime(0.025, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.035);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(now);
    osc.stop(now + 0.04);
  }
}

function playComposerFoley(fxType, semitoneOffset = 0, soundPath = null) {
  if (!sfxEnabled) return;
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const rate = Math.max(0.55, Math.min(1.85, Math.pow(2, semitoneOffset / 12)));
  const now = ctx.currentTime;

  const targetPath = soundPath || (fxType === 'pig' ? 'mob/pig/say1.ogg' : (fxType === 'anvil' ? 'random/anvil_land.ogg' : 'random/orb.ogg'));
  const buf = DECODED_BUFFER_CACHE[targetPath];
  if (buf) {
    const src = ctx.createBufferSource();
    const g = ctx.createGain();
    src.buffer = buf;
    src.playbackRate.setValueAtTime(rate, now);
    g.gain.setValueAtTime(0.24, now);
    g.gain.exponentialRampToValueAtTime(0.001, now + 0.28);
    src.connect(g);
    g.connect(ctx.destination);
    src.start(now);
    src.stop(now + 0.3);
    return;
  }

  // Immediate pitch-locked synthesis for Foley accent
  const osc = ctx.createOscillator();
  const g = ctx.createGain();
  const baseF = 369.99 * rate;
  osc.type = fxType === 'anvil' ? 'square' : (fxType === 'pig' ? 'sawtooth' : 'sine');
  osc.frequency.setValueAtTime(baseF, now);
  osc.frequency.exponentialRampToValueAtTime(fxType === 'ender' ? baseF * 1.5 : baseF * 1.25, now + 0.14);
  g.gain.setValueAtTime(0.08, now);
  g.gain.exponentialRampToValueAtTime(0.001, now + 0.16);
  osc.connect(g);
  g.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + 0.17);
}

// ==============================================================================
// 5B. Offline Multi-Track PCM Renderer + Genuine Xiph.org Ogg Vorbis I (.ogg)
//     Encoder + Minecraft Resource Pack (.zip) Builder
// ==============================================================================

/**
 * Renders `numBars` of the current stochastic `disc_maker` composition at 44,100 Hz Mono
 * (mixing decoded Minecraft `.ogg` buffers + pitch-locked physical modeling + drums + foley).
 */
function renderDiscMakerToMonoPcm(numBars = 4, sampleRate = 44100) {
  const bpm = parseInt(document.getElementById('dm-bpm-slider')?.value || '114', 10) || 114;
  const swingPct = (parseInt(document.getElementById('dm-swing-slider')?.value || '20', 10) || 20) / 100;
  const scaleKey = document.getElementById('dm-scale-select')?.value || 'phrygian';
  const scale = DM_SCALES[scaleKey] || DM_SCALES.phrygian;
  const rootOffset = parseInt(document.getElementById('dm-root-select')?.value || '0', 10) || 0;
  const harmonicLock = (parseInt(document.getElementById('dm-harmonic-lock-slider')?.value || '88', 10) || 88) / 100;

  const sixteenthSec = (60 / bpm) / 4;
  const totalSteps = numBars * 16;
  const totalDurationSec = totalSteps * sixteenthSec + 0.6;
  const totalSamples = Math.ceil(totalDurationSec * sampleRate);
  const mix = new Float32Array(totalSamples);

  let cursorSec = 0.05;
  for (let bar = 0; bar < numBars; bar++) {
    const activeChordDegree = dmCurrentChords[bar % dmCurrentChords.length] || 0;
    for (let step = 0; step < 16; step++) {
      const stepDur = (step % 2 === 0)
        ? sixteenthSec * (1 + swingPct * 0.5)
        : sixteenthSec * (1 - swingPct * 0.5);
      const startSample = Math.floor(cursorSec * sampleRate);

      for (let t = 0; t < dmTracks.length; t++) {
        const cellData = dmPattern[t][step];
        if (!cellData) continue;
        const baseTrack = dmTracks[t];
        const activeVoice = cellData.customVoice ? { ...baseTrack, ...cellData.customVoice, synthType: cellData.customVoice.synth } : baseTrack;

        if (baseTrack.id === 'drums') {
          mixDrumIntoPcm(mix, startSample, sampleRate, cellData.drumType);
        } else if (baseTrack.id === 'fx') {
          const foleySemi = rootOffset + (scale[(activeChordDegree + (cellData.degree || 0)) % scale.length] || 0);
          mixFoleyIntoPcm(mix, startSample, sampleRate, cellData.fxType, foleySemi, cellData.sound);
        } else {
          const totalDegree = activeChordDegree + (cellData.degree || 0);
          const scaleLen = scale.length;
          const octaveFromDegree = Math.floor(totalDegree / scaleLen);
          const scaleSemitone = scale[((totalDegree % scaleLen) + scaleLen) % scaleLen];
          const semitoneOffset = rootOffset + scaleSemitone + (octaveFromDegree + (cellData.octave || 0)) * 12;
          mixPitchedMinecraftVoiceIntoPcm(mix, startSample, sampleRate, activeVoice, semitoneOffset, harmonicLock);
        }
      }
      cursorSec += stepDur;
    }
  }

  // Normalize and soft-clip to -0.8 dBFS for warm Minecraft Jukebox playback
  let peak = 0.001;
  for (let i = 0; i < mix.length; i++) {
    const a = Math.abs(mix[i]);
    if (a > peak) peak = a;
  }
  const normGain = 0.88 / peak;
  for (let i = 0; i < mix.length; i++) {
    mix[i] = Math.tanh(mix[i] * normGain * 1.15);
  }
  return mix;
}

function mixPitchedMinecraftVoiceIntoPcm(mix, startSample, sampleRate, track, semitoneOffset, harmonicLock) {
  const baseRate = track.baseRate || 1.0;
  const exactRate = baseRate * Math.pow(2, semitoneOffset / 12);
  const clampedRate = Math.max(0.35, Math.min(2.8, exactRate * harmonicLock + baseRate * (1 - harmonicLock)));
  const gateDecay = track.id === 'bass' ? 0.30 : (track.id === 'bell' ? 0.38 : 0.25);
  const durSamples = Math.floor(gateDecay * sampleRate);

  // 1. Mix decoded Minecraft .ogg sample if available in DECODED_BUFFER_CACHE
  const buf = DECODED_BUFFER_CACHE[track.sound];
  if (buf) {
    const chData = buf.getChannelData(0);
    const bufRateRatio = (buf.sampleRate / sampleRate) * clampedRate;
    const amp = track.id === 'bass' ? 0.42 : 0.30;
    const maxS = Math.min(durSamples, Math.floor(chData.length / bufRateRatio));
    for (let i = 0; i < maxS; i++) {
      const dstIdx = startSample + i;
      if (dstIdx >= mix.length) break;
      const srcPos = i * bufRateRatio;
      const idx0 = Math.floor(srcPos);
      const frac = srcPos - idx0;
      const s0 = chData[idx0] || 0;
      const s1 = chData[idx0 + 1] || s0;
      const env = Math.exp(-3.8 * (i / durSamples));
      mix[dstIdx] += (s0 + (s1 - s0) * frac) * amp * env;
    }
  }

  // 2. Mix pitch-locked Minecraft timbre resonator / physical model
  const baseHz = track.id === 'bass' ? 92.5 : (track.id === 'bell' ? 739.99 : 369.99);
  const freq = baseHz * Math.pow(2, semitoneOffset / 12);
  const timbre = track.timbre || 'pling';
  const synthAmp = (track.id === 'bass' ? 0.26 : 0.18) * (buf ? 0.55 * harmonicLock : 1.0);

  let phase1 = 0;
  let phase2 = 0;
  for (let i = 0; i < durSamples; i++) {
    const dstIdx = startSample + i;
    if (dstIdx >= mix.length) break;
    const t = i / sampleRate;
    const env = Math.min(1, i / (0.004 * sampleRate)) * Math.exp(-4.5 * (t / gateDecay));

    let f1 = freq;
    let f2 = freq * 2.0;
    if (timbre === 'enderman' || timbre === 'portal') {
      f1 = freq * (1 + 0.06 * Math.exp(-t * 22));
      f2 = freq * 1.5;
    } else if (timbre === 'villager' || timbre === 'cat' || timbre === 'pig' || timbre === 'ghast') {
      f1 = freq * (1 + 0.03 * Math.sin(2 * Math.PI * 6.5 * t));
      f2 = freq * 3.0;
    } else if (timbre === 'anvil' || timbre === 'glass') {
      f2 = freq * 2.756; // Metallic bell overtone
    } else if (timbre === 'slime' || timbre === 'lava' || timbre === 'piston') {
      f1 = freq * (1 + 0.28 * Math.exp(-t * 28));
      f2 = freq * 0.5;
    }

    phase1 += (2 * Math.PI * f1) / sampleRate;
    phase2 += (2 * Math.PI * f2) / sampleRate;

    let w1 = Math.sin(phase1);
    if (track.synthType === 'square') w1 = w1 >= 0 ? 0.75 : -0.75;
    else if (track.synthType === 'sawtooth') w1 = 2 * ((phase1 / (2 * Math.PI)) - Math.floor(0.5 + phase1 / (2 * Math.PI)));

    const w2 = Math.sin(phase2) * 0.35;
    mix[dstIdx] += (w1 * 0.7 + w2) * synthAmp * env;
  }
}

function mixDrumIntoPcm(mix, startSample, sampleRate, drumType) {
  if (drumType === 'kick') {
    const dur = Math.floor(0.16 * sampleRate);
    let phase = 0;
    for (let i = 0; i < dur; i++) {
      const dst = startSample + i;
      if (dst >= mix.length) break;
      const t = i / sampleRate;
      const f = 38 + 105 * Math.exp(-t * 26);
      phase += (2 * Math.PI * f) / sampleRate;
      const env = Math.exp(-t * 18);
      mix[dst] += Math.sin(phase) * 0.42 * env;
    }
  } else if (drumType === 'snare') {
    const dur = Math.floor(0.13 * sampleRate);
    let phase = 0;
    for (let i = 0; i < dur; i++) {
      const dst = startSample + i;
      if (dst >= mix.length) break;
      const t = i / sampleRate;
      const f = 95 + 125 * Math.exp(-t * 30);
      phase += (2 * Math.PI * f) / sampleRate;
      const noise = (Math.sin(i * 12.9898) * 43758.5453) % 1;
      const env = Math.exp(-t * 24);
      mix[dst] += (Math.sin(phase) * 0.45 + noise * 0.55) * 0.28 * env;
    }
  } else {
    const dur = Math.floor(0.05 * sampleRate);
    for (let i = 0; i < dur; i++) {
      const dst = startSample + i;
      if (dst >= mix.length) break;
      const t = i / sampleRate;
      const noise = ((Math.sin(i * 78.233) * 43758.5453) % 1) * (i % 2 === 0 ? 1 : -1);
      const env = Math.exp(-t * 65);
      mix[dst] += noise * 0.14 * env;
    }
  }
}

function mixFoleyIntoPcm(mix, startSample, sampleRate, fxType, semitoneOffset, soundPath) {
  const rate = Math.max(0.55, Math.min(1.85, Math.pow(2, semitoneOffset / 12)));
  const targetPath = soundPath || (fxType === 'pig' ? 'mob/pig/say1.ogg' : (fxType === 'anvil' ? 'random/anvil_land.ogg' : 'random/orb.ogg'));
  const buf = DECODED_BUFFER_CACHE[targetPath];
  if (buf) {
    const chData = buf.getChannelData(0);
    const ratio = (buf.sampleRate / sampleRate) * rate;
    const maxS = Math.min(Math.floor(0.28 * sampleRate), Math.floor(chData.length / ratio));
    for (let i = 0; i < maxS; i++) {
      const dst = startSample + i;
      if (dst >= mix.length) break;
      const srcIdx = Math.floor(i * ratio);
      const env = Math.exp(-4.5 * (i / maxS));
      mix[dst] += (chData[srcIdx] || 0) * 0.28 * env;
    }
    return;
  }
  const dur = Math.floor(0.16 * sampleRate);
  let phase = 0;
  const baseF = 369.99 * rate;
  for (let i = 0; i < dur; i++) {
    const dst = startSample + i;
    if (dst >= mix.length) break;
    const t = i / sampleRate;
    const f = baseF * (1 + 0.25 * (t / 0.16));
    phase += (2 * Math.PI * f) / sampleRate;
    const env = Math.exp(-t * 18);
    mix[dst] += Math.sin(phase) * 0.18 * env;
  }
}

// Precomputed Ogg CRC-32 Lookup Table (Polynomial 0x04C11DB7)
const OGG_CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let i = 0; i < 256; i++) {
    let r = i << 24;
    for (let j = 0; j < 8; j++) {
      r = (r & 0x80000000) ? ((r << 1) ^ 0x04c11db7) : (r << 1);
    }
    table[i] = r >>> 0;
  }
  return table;
})();

function computeOggCrc32(bytes) {
  let crc = 0;
  for (let i = 0; i < bytes.length; i++) {
    crc = ((crc << 8) ^ OGG_CRC_TABLE[((crc >>> 24) ^ bytes[i]) & 0xff]) >>> 0;
  }
  return crc >>> 0;
}

function buildOggPageFromPackets(packets, headerType, granulePos, serialNo, pageSeqNo) {
  const segTable = [];
  let totalPayloadBytes = 0;
  for (const pkt of packets) {
    let rem = pkt.length;
    while (rem >= 255) {
      segTable.push(255);
      rem -= 255;
    }
    segTable.push(rem);
    totalPayloadBytes += pkt.length;
  }

  const headerLen = 27 + segTable.length;
  const page = new Uint8Array(headerLen + totalPayloadBytes);
  const view = new DataView(page.buffer);

  // 'OggS' capture pattern
  page[0] = 0x4f; page[1] = 0x67; page[2] = 0x67; page[3] = 0x53;
  page[4] = 0; // stream_structure_version
  page[5] = headerType;
  // 64-bit granule_position (little-endian)
  view.setUint32(6, granulePos >>> 0, true);
  view.setUint32(10, Math.floor(granulePos / 4294967296) >>> 0, true);
  view.setUint32(14, serialNo >>> 0, true);
  view.setUint32(18, pageSeqNo >>> 0, true);
  view.setUint32(22, 0, true); // CRC placeholder
  page[26] = segTable.length;
  for (let i = 0; i < segTable.length; i++) {
    page[27 + i] = segTable[i];
  }

  let offset = headerLen;
  for (const pkt of packets) {
    page.set(pkt, offset);
    offset += pkt.length;
  }

  const crc = computeOggCrc32(page);
  view.setUint32(22, crc, true);
  return page;
}

function vorbisFloat32Pack(val) {
  if (val === 0) return 0;
  let sign = 0;
  if (val < 0) {
    sign = 0x80000000;
    val = -val;
  }
  let exp = Math.floor(Math.log2(val));
  let mant = Math.round(val / Math.pow(2, exp - 20));
  if (mant >= 0x200000) {
    mant >>>= 1;
    exp += 1;
  }
  return (sign | ((exp + 788 - 20) << 21) | (mant & 0x1fffff)) >>> 0;
}

class VorbisBitWriter {
  constructor() {
    this.bytes = [];
    this.cur = 0;
    this.nbits = 0;
  }
  write(val, bits) {
    for (let i = 0; i < bits; i++) {
      if ((val >>> i) & 1) {
        this.cur |= (1 << this.nbits);
      }
      this.nbits++;
      if (this.nbits === 8) {
        this.bytes.push(this.cur);
        this.cur = 0;
        this.nbits = 0;
      }
    }
  }
  writeAscii(str) {
    for (let i = 0; i < str.length; i++) {
      this.write(str.charCodeAt(i), 8);
    }
  }
  finish() {
    if (this.nbits > 0) {
      this.bytes.push(this.cur);
      this.cur = 0;
      this.nbits = 0;
    }
    return new Uint8Array(this.bytes);
  }
}

// Precomputed 256-sample Vorbis window & MDCT cosine matrix for fast in-browser encoding
const VORBIS_N = 256;
const VORBIS_N2 = 128;
const VORBIS_WIN = (() => {
  const w = new Float32Array(VORBIS_N);
  for (let i = 0; i < VORBIS_N; i++) {
    const s = Math.sin(((i + 0.5) / VORBIS_N) * Math.PI);
    w[i] = Math.sin(0.5 * Math.PI * s * s);
  }
  return w;
})();

const VORBIS_COS_TABLE = (() => {
  const tbl = new Float32Array(VORBIS_N2 * VORBIS_N);
  for (let k = 0; k < VORBIS_N2; k++) {
    for (let n = 0; n < VORBIS_N; n++) {
      tbl[k * VORBIS_N + n] = Math.cos((Math.PI / VORBIS_N) * (n + 0.5 + VORBIS_N / 4) * (k + 0.5));
    }
  }
  return tbl;
})();

/**
 * Encodes 44.1 kHz Mono Float32 PCM samples into a 100% compliant Xiph.org Ogg Vorbis I (.ogg) file
 * compatible with Minecraft Jukeboxes (stb_vorbis / OpenAL), ffmpeg, VLC, and browser <audio>.
 */
function encodeMonoPcmToOggVorbis(samples, sampleRate = 44100) {
  const serialNo = 0x53415742;
  const pages = [];
  let seqNo = 0;

  // 1. Identification Header (\x01vorbis)
  const idHdr = new Uint8Array(30);
  const idView = new DataView(idHdr.buffer);
  idHdr[0] = 1;
  idHdr.set([118, 111, 114, 98, 105, 115], 1); // 'vorbis'
  idView.setUint32(7, 0, true); // version = 0
  idHdr[11] = 1; // channels = 1 (Mono for positional Minecraft Jukebox audio)
  idView.setUint32(12, sampleRate, true);
  idView.setInt32(16, 0, true);
  idView.setInt32(20, 128000, true);
  idView.setInt32(24, 0, true);
  idHdr[28] = 0x88; // blocksize_0 = 256 (2^8), blocksize_1 = 256 (2^8)
  idHdr[29] = 1;    // framing_flag = 1
  pages.push(buildOggPageFromPackets([idHdr], 0x02, 0, serialNo, seqNo++));

  // 2. Comment Header (\x03vorbis)
  const vendor = 'SawbyteDiscMaker26.3';
  const cmtHdr = new Uint8Array(7 + 4 + vendor.length + 4 + 1);
  const cmtView = new DataView(cmtHdr.buffer);
  cmtHdr[0] = 3;
  cmtHdr.set([118, 111, 114, 98, 105, 115], 1);
  cmtView.setUint32(7, vendor.length, true);
  for (let i = 0; i < vendor.length; i++) cmtHdr[11 + i] = vendor.charCodeAt(i);
  cmtView.setUint32(11 + vendor.length, 0, true);
  cmtHdr[11 + vendor.length + 4] = 1;
  pages.push(buildOggPageFromPackets([cmtHdr], 0x00, 0, serialNo, seqNo++));

  // 3. Setup Header (\x05vorbis)
  const bw = new VorbisBitWriter();
  bw.write(5, 8);
  bw.writeAscii('vorbis');
  bw.write(1, 8); // 2 codebooks (count - 1 = 1)

  // Codebook 0: 2 entries, dim 1, lengths [1, 1], lookup_type = 0 (classbook)
  bw.write(0x564342, 24);
  bw.write(1, 16);
  bw.write(2, 24);
  bw.write(0, 1);
  bw.write(0, 1);
  bw.write(0, 5);
  bw.write(0, 5);
  bw.write(0, 4);

  // Codebook 1: 256 entries, dim 1, uniform 8-bit codewords, lookup_type = 1 (-1.5 .. +1.5)
  bw.write(0x564342, 24);
  bw.write(1, 16);
  bw.write(256, 24);
  bw.write(0, 1);
  bw.write(0, 1);
  for (let i = 0; i < 256; i++) bw.write(7, 5);
  bw.write(1, 4);
  bw.write(vorbisFloat32Pack(-1.5), 32);
  bw.write(vorbisFloat32Pack(3.0 / 255.0), 32);
  bw.write(7, 4);
  bw.write(0, 1);
  for (let i = 0; i < 256; i++) bw.write(i, 8);

  // Time domain transforms = 1 (0)
  bw.write(0, 6);
  bw.write(0, 16);

  // Floor 0: Floor1 with 0 partitions (flat unity floor)
  bw.write(0, 6);
  bw.write(1, 16);
  bw.write(0, 5);
  bw.write(0, 2);
  bw.write(4, 4);

  // Residue 0: Residue1 over 0..128 using Codebook 1
  bw.write(0, 6);
  bw.write(1, 16);
  bw.write(0, 24);
  bw.write(128, 24);
  bw.write(127, 24);
  bw.write(0, 6);
  bw.write(0, 8);
  bw.write(1, 3);
  bw.write(0, 1);
  bw.write(1, 8);

  // Mapping 0
  bw.write(0, 6);
  bw.write(0, 16);
  bw.write(0, 1);
  bw.write(0, 1);
  bw.write(0, 2);
  bw.write(0, 8);
  bw.write(0, 8);
  bw.write(0, 8);

  // Mode 0
  bw.write(0, 6);
  bw.write(0, 1);
  bw.write(0, 16);
  bw.write(0, 16);
  bw.write(0, 8);
  bw.write(1, 1);

  const setupHdr = bw.finish();
  pages.push(buildOggPageFromPackets([setupHdr], 0x00, 0, serialNo, seqNo++));

  // 4. Encode Audio Frames (hop = 128 samples, 32 packets per Ogg page)
  const totalFrames = Math.max(2, Math.floor((samples.length - VORBIS_N) / VORBIS_N2));
  const windowed = new Float32Array(VORBIS_N);
  let batchPackets = [];
  let granulePos = 0;

  for (let f = 0; f < totalFrames; f++) {
    const baseIdx = f * VORBIS_N2;
    for (let n = 0; n < VORBIS_N; n++) {
      windowed[n] = (samples[baseIdx + n] || 0) * VORBIS_WIN[n];
    }

    const fbw = new VorbisBitWriter();
    fbw.write(0, 1);   // packet_type = 0 (audio)
    fbw.write(1, 1);   // floor nonzero = 1
    fbw.write(255, 8); // Y0 = 255 (0 dB unity floor)
    fbw.write(255, 8); // Y1 = 255 (0 dB unity floor)
    fbw.write(0, 1);   // residue classword = 0

    for (let k = 0; k < VORBIS_N2; k++) {
      let sum = 0;
      const rowOffset = k * VORBIS_N;
      for (let n = 0; n < VORBIS_N; n++) {
        sum += windowed[n] * VORBIS_COS_TABLE[rowOffset + n];
      }
      const normMdct = sum / 128.0;
      let qIdx = Math.round(((normMdct + 1.5) / 3.0) * 255.0);
      if (qIdx < 0) qIdx = 0;
      else if (qIdx > 255) qIdx = 255;
      fbw.write(qIdx, 8);
    }

    batchPackets.push(fbw.finish());
    granulePos = (f + 1) * VORBIS_N2;

    const isLast = (f === totalFrames - 1);
    if (batchPackets.length >= 32 || isLast) {
      pages.push(buildOggPageFromPackets(batchPackets, isLast ? 0x04 : 0x00, granulePos, serialNo, seqNo++));
      batchPackets = [];
    }
  }

  let totalLen = 0;
  for (const p of pages) totalLen += p.length;
  const out = new Uint8Array(totalLen);
  let pos = 0;
  for (const p of pages) {
    out.set(p, pos);
    pos += p.length;
  }
  return out;
}

// Standard IEEE 802.3 CRC-32 for .ZIP archive generation
const ZIP_CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let i = 0; i < 256; i++) {
    let c = i;
    for (let k = 0; k < 8; k++) {
      c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
    }
    table[i] = c >>> 0;
  }
  return table;
})();

function computeZipCrc32(bytes) {
  let crc = 0xffffffff;
  for (let i = 0; i < bytes.length; i++) {
    crc = (crc >>> 8) ^ ZIP_CRC_TABLE[(crc ^ bytes[i]) & 0xff];
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function buildUncompressedZipArchive(files) {
  const enc = new TextEncoder();
  const localParts = [];
  const centralParts = [];
  let offset = 0;

  for (const file of files) {
    const nameBytes = enc.encode(file.path);
    const dataBytes = file.data instanceof Uint8Array ? file.data : enc.encode(String(file.data));
    const crc = computeZipCrc32(dataBytes);

    const localHdr = new Uint8Array(30 + nameBytes.length);
    const lv = new DataView(localHdr.buffer);
    lv.setUint32(0, 0x04034b50, true);
    lv.setUint16(4, 20, true);
    lv.setUint16(6, 0, true);
    lv.setUint16(8, 0, true); // store (uncompressed)
    lv.setUint16(10, 0, true);
    lv.setUint16(12, 0x5421, true);
    lv.setUint32(14, crc, true);
    lv.setUint32(18, dataBytes.length, true);
    lv.setUint32(22, dataBytes.length, true);
    lv.setUint16(26, nameBytes.length, true);
    lv.setUint16(28, 0, true);
    localHdr.set(nameBytes, 30);

    const centralHdr = new Uint8Array(46 + nameBytes.length);
    const cv = new DataView(centralHdr.buffer);
    cv.setUint32(0, 0x02014b50, true);
    cv.setUint16(4, 20, true);
    cv.setUint16(6, 20, true);
    cv.setUint16(8, 0, true);
    cv.setUint16(10, 0, true);
    cv.setUint16(12, 0, true);
    cv.setUint16(14, 0x5421, true);
    cv.setUint32(16, crc, true);
    cv.setUint32(20, dataBytes.length, true);
    cv.setUint32(24, dataBytes.length, true);
    cv.setUint16(28, nameBytes.length, true);
    cv.setUint16(30, 0, true);
    cv.setUint16(32, 0, true);
    cv.setUint16(34, 0, true);
    cv.setUint16(36, 0, true);
    cv.setUint32(38, 0, true);
    cv.setUint32(42, offset, true);
    centralHdr.set(nameBytes, 46);

    localParts.push(localHdr, dataBytes);
    centralParts.push(centralHdr);
    offset += localHdr.length + dataBytes.length;
  }

  let centralSize = 0;
  for (const c of centralParts) centralSize += c.length;

  const eocd = new Uint8Array(22);
  const ev = new DataView(eocd.buffer);
  ev.setUint32(0, 0x06054b50, true);
  ev.setUint16(4, 0, true);
  ev.setUint16(6, 0, true);
  ev.setUint16(8, files.length, true);
  ev.setUint16(10, files.length, true);
  ev.setUint32(12, centralSize, true);
  ev.setUint32(16, offset, true);
  ev.setUint16(20, 0, true);

  const total = offset + centralSize + 22;
  const out = new Uint8Array(total);
  let pos = 0;
  for (const p of localParts) { out.set(p, pos); pos += p.length; }
  for (const p of centralParts) { out.set(p, pos); pos += p.length; }
  out.set(eocd, pos);
  return out;
}

function exportDiscMakerAsOggFile(asResourcePack = false) {
  const targetDisc = document.getElementById('dm-target-disc-select')?.value || 'pigstep';
  const numBars = parseInt(document.getElementById('dm-export-bars-select')?.value || '4', 10) || 4;
  const statusEl = document.getElementById('dm-export-status-text');
  if (statusEl) {
    statusEl.innerHTML = `<span class="mc-e">Rendering ${numBars} bars of stochastic Minecraft audio to Ogg Vorbis (${targetDisc}.ogg)...</span>`;
  }

  setTimeout(() => {
    try {
      const pcm = renderDiscMakerToMonoPcm(numBars, 44100);
      const oggBytes = encodeMonoPcmToOggVorbis(pcm, 44100);
      const oggBlob = new Blob([oggBytes], { type: 'audio/ogg' });

      if (lastExportedOggBlobUrl) {
        URL.revokeObjectURL(lastExportedOggBlobUrl);
      }
      lastExportedOggBlobUrl = URL.createObjectURL(oggBlob);

      const previewWrap = document.getElementById('dm-exported-preview-wrap');
      const previewAudio = document.getElementById('dm-exported-audio-preview');
      const previewLabel = document.getElementById('dm-exported-preview-label');
      if (previewWrap && previewAudio) {
        previewWrap.style.display = 'flex';
        previewAudio.src = lastExportedOggBlobUrl;
        if (previewLabel) {
          previewLabel.textContent = `Rendered ${targetDisc}.ogg (${(oggBytes.length / 1024).toFixed(1)} KB, 44.1 kHz Mono Ogg Vorbis):`;
        }
      }

      if (asResourcePack) {
        const mcmeta = JSON.stringify({
          pack: {
            pack_format: 34,
            supported_formats: [15, 64],
            description: `§bSawbyte disc_maker Custom Disc (§e${targetDisc}.ogg§b)`
          }
        }, null, 2);

        const readme = [
          `Sawbyte disc_maker — Custom Minecraft Music Disc Resource Pack`,
          `==============================================================`,
          `Replaced Music Disc: assets/minecraft/sounds/records/${targetDisc}.ogg`,
          `Format: Ogg Vorbis I (44,100 Hz Mono — 100% compatible with Minecraft Jukeboxes)`,
          ``,
          `How to listen in Minecraft:`,
          `1. Drop this .zip file directly into your Minecraft 'resourcepacks' folder`,
          `   (or ~/.sawbyte/instances/<instance>/resourcepacks/).`,
          `2. In Minecraft, open Options -> Resource Packs and enable this pack.`,
          `3. Insert the '${targetDisc}' Music Disc into any Jukebox in-game!`
        ].join('\n');

        const zipBytes = buildUncompressedZipArchive([
          { path: 'pack.mcmeta', data: mcmeta },
          { path: `assets/minecraft/sounds/records/${targetDisc}.ogg`, data: oggBytes },
          { path: 'README.txt', data: readme }
        ]);
        const zipBlob = new Blob([zipBytes], { type: 'application/zip' });
        const zipUrl = URL.createObjectURL(zipBlob);
        const a = document.createElement('a');
        a.href = zipUrl;
        a.download = `Sawbyte_DiscPack_${targetDisc}.zip`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        setTimeout(() => URL.revokeObjectURL(zipUrl), 5000);

        if (statusEl) {
          statusEl.innerHTML = `<span class="mc-a">Exported <code>Sawbyte_DiscPack_${targetDisc}.zip</code> (contains <code>assets/minecraft/sounds/records/${targetDisc}.ogg</code>)! Drop into <code>resourcepacks/</code> to play in a Jukebox!</span>`;
        }
        showAdvancementToast('Resource Pack Exported!', `Sawbyte_DiscPack_${targetDisc}.zip ready for Minecraft`, 'block/jukebox_side.png');
      } else {
        const a = document.createElement('a');
        a.href = lastExportedOggBlobUrl;
        a.download = `${targetDisc}.ogg`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);

        if (statusEl) {
          statusEl.innerHTML = `<span class="mc-a">Exported <code>${targetDisc}.ogg</code> (${(oggBytes.length / 1024).toFixed(1)} KB, Ogg Vorbis 44.1 kHz Mono)! Replace <code>assets/minecraft/sounds/records/${targetDisc}.ogg</code> in any Resource Pack!</span>`;
        }
        showAdvancementToast('Exported .OGG Music Disc!', `Saved ${targetDisc}.ogg for Minecraft Jukeboxes`, 'item/music_disc_pigstep.png');
      }
    } catch (err) {
      if (statusEl) {
        statusEl.innerHTML = `<span class="mc-c">Export error: ${err.message || err}</span>`;
      }
    }
  }, 30);
}

// ==============================================================================
// 6. Official Minecraft Audio Engine & Pigstep Jukebox
// ==============================================================================

function getSoundUrls(soundRelativePath) {
  const clean = soundRelativePath.replace(/^\/+/, '');
  const fullKey = `minecraft/sounds/${clean}`;
  const urls = [];

  for (const br of [activeMcAssetBranch, ...MC_ASSET_BRANCH_CANDIDATES]) {
    urls.push(`https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/${br}/assets/minecraft/sounds/${clean}`);
    urls.push(`https://cdn.jsdelivr.net/gh/InventivetalentDev/minecraft-assets@${br}/assets/minecraft/sounds/${clean}`);
    urls.push(`https://assets.mcasset.cloud/${br}/assets/minecraft/sounds/${clean}`);
  }

  const sha1 = SOUND_SHA1_CACHE[fullKey];
  if (sha1) {
    urls.push(`https://resources.download.minecraft.net/${sha1.slice(0, 2)}/${sha1}`);
  }

  return urls;
}

let lastGlobalSfxTime = 0;
const MIN_SFX_GAP_MS = 85;

function playMinecraftOgg(soundRelativePath, volume = 0.45, playbackRate = 1.0, fallbackFn = null) {
  if (!sfxEnabled) return;
  const now = performance.now();
  if (now - lastGlobalSfxTime < MIN_SFX_GAP_MS) return;
  lastGlobalSfxTime = now;

  // Immediate, zero-latency WebAudio synthesis (eliminates 404 network cascades & multi-trigger spam)
  if (typeof fallbackFn === 'function') {
    fallbackFn();
    return;
  }
  const p = String(soundRelativePath || '');
  if (p.includes('orb') || p.includes('levelup') || p.includes('totem') || p.includes('bell')) {
    synthXpFallback();
  } else if (p.includes('pig')) {
    synthPigFallback();
  } else {
    synthClickFallback(playbackRate || 1.0);
  }
}

function ensureAudioCtx() {
  if (!sfxEnabled) return null;
  if (!audioCtx) {
    const AudioContextClass = window.AudioContext || window.webkitAudioContext;
    if (AudioContextClass) audioCtx = new AudioContextClass();
  }
  if (audioCtx && audioCtx.state === 'suspended') {
    audioCtx.resume().catch(() => {});
  }
  return audioCtx;
}

function synthClickFallback(pitch = 1.0) {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = 'triangle';
  osc.frequency.setValueAtTime(190 * pitch, now);
  osc.frequency.exponentialRampToValueAtTime(65 * pitch, now + 0.045);
  gain.gain.setValueAtTime(0.08, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.045);
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + 0.05);
}

function synthXpFallback() {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;
  [587.33, 783.99, 987.77, 1174.66].forEach((f, idx) => {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'sine';
    const start = now + idx * 0.045;
    osc.frequency.setValueAtTime(f, start);
    gain.gain.setValueAtTime(0.06, start);
    gain.gain.exponentialRampToValueAtTime(0.001, start + 0.22);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(start);
    osc.stop(start + 0.24);
  });
}

function synthPigFallback() {
  const ctx = ensureAudioCtx();
  if (!ctx) return;
  const now = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = 'sawtooth';
  osc.frequency.setValueAtTime(145, now);
  osc.frequency.linearRampToValueAtTime(210, now + 0.09);
  osc.frequency.exponentialRampToValueAtTime(95, now + 0.24);
  gain.gain.setValueAtTime(0.08, now);
  gain.gain.exponentialRampToValueAtTime(0.001, now + 0.25);
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(now);
  osc.stop(now + 0.26);
}

function playMcClick(pitch = 1.0) {
  playMinecraftOgg('random/click.ogg', 0.35, pitch, () => synthClickFallback(pitch));
}

function playMcXpChime() {
  playMinecraftOgg('random/orb.ogg', 0.4, 1.0 + (Math.random() * 0.2 - 0.1), () => synthXpFallback());
}

function playMcLevelUp() {
  playMinecraftOgg('random/levelup.ogg', 0.45, 1.0, () => synthXpFallback());
}

function playMcPigSound() {
  const variant = Math.floor(Math.random() * 3) + 1;
  playMinecraftOgg(`mob/pig/say${variant}.ogg`, 0.55, 0.95 + Math.random() * 0.12, () => synthPigFallback());
}

function playMcTotemSound() {
  playMinecraftOgg('item/totem/use_totem.ogg', 0.5, 1.0, () => synthXpFallback());
}

function playMcBellSound() {
  playMinecraftOgg('note/bell.ogg', 0.55, 1.0, () => synthXpFallback());
}

function playMcEatSound() {
  const variant = Math.floor(Math.random() * 3) + 1;
  playMinecraftOgg(`random/eat${variant}.ogg`, 0.45, 1.0, () => synthClickFallback(1.2));
}

function playMcFireworkSound() {
  playMinecraftOgg('fireworks/launch1.ogg', 0.45, 1.0, () => synthXpFallback());
  setTimeout(() => {
    playMinecraftOgg('fireworks/twinkle1.ogg', 0.4, 1.0);
  }, 550);
}

function initSoundEngine() {
  const toggleBtn = document.getElementById('sfx-toggle-btn');
  const toggleLabel = document.getElementById('sfx-toggle-label');
  if (toggleBtn) {
    toggleBtn.addEventListener('click', () => {
      sfxEnabled = !sfxEnabled;
      toggleBtn.classList.toggle('muted', !sfxEnabled);
      if (toggleLabel) toggleLabel.textContent = sfxEnabled ? 'SFX: ON' : 'SFX: OFF';
      if (sfxEnabled) playMcClick(1.1);
    });
  }

  document.querySelectorAll('.sfx-trigger').forEach((el) => {
    el.addEventListener('click', () => playMcClick(1.0));
  });

  document.querySelectorAll('.sfx-xp').forEach((el) => {
    el.addEventListener('click', () => playMcXpChime());
  });
}

function initPigstepJukebox() {
  bgMusicAudio = new Audio();
  bgMusicAudio.crossOrigin = 'anonymous';
  bgMusicAudio.loop = true;
  bgMusicAudio.volume = bgMusicVolume;

  const playBtn = document.getElementById('jukebox-play-btn');
  const toggleBtn = document.getElementById('jukebox-toggle-btn');
  const nextBtn = document.getElementById('jukebox-next-btn');
  const volSlider = document.getElementById('jukebox-volume');

  loadDiscIntoPlayer(0, true);

  const unlockAutoplay = () => {
    if (!bgMusicPlaying && !userPausedMusic && !dmPlaying) {
      bgMusicAudio.volume = bgMusicVolume;
      bgMusicAudio.play().then(() => {
        bgMusicPlaying = true;
        updateJukeboxUI();
      }).catch(() => {});
    }
  };

  // Jukebox only plays when user explicitly clicks Play (no surprise autoplay on page click)
  userPausedMusic = true;

  const toggleMusic = () => {
    if (!bgMusicAudio) return;
    if (dmPlaying) stopDiscMaker();
    if (bgMusicPlaying) {
      bgMusicAudio.pause();
      bgMusicPlaying = false;
      userPausedMusic = true;
    } else {
      userPausedMusic = false;
      bgMusicAudio.volume = bgMusicVolume;
      bgMusicAudio.play().then(() => {
        bgMusicPlaying = true;
        updateJukeboxUI();
      }).catch(() => {
        loadDiscIntoPlayer(currentDiscIndex, true);
      });
    }
    updateJukeboxUI();
  };

  if (playBtn) playBtn.addEventListener('click', toggleMusic);
  if (toggleBtn) toggleBtn.addEventListener('click', toggleMusic);

  if (nextBtn) {
    nextBtn.addEventListener('click', () => {
      if (dmPlaying) stopDiscMaker();
      currentDiscIndex = (currentDiscIndex + 1) % JUKEBOX_DISCS.length;
      userPausedMusic = false;
      loadDiscIntoPlayer(currentDiscIndex, true);
      const disc = JUKEBOX_DISCS[currentDiscIndex];
      showAdvancementToast('Disc Changed', disc.title, disc.tex);
      playMcClick(1.15);
    });
  }

  if (volSlider) {
    volSlider.addEventListener('input', (e) => {
      const pct = parseInt(e.target.value, 10) || 0;
      bgMusicVolume = pct / 100;
      if (bgMusicAudio) bgMusicAudio.volume = bgMusicVolume;
      updateJukeboxUI();
    });
  }
}

function loadDiscIntoPlayer(discIdx, autoPlay = true) {
  const disc = JUKEBOX_DISCS[discIdx] || JUKEBOX_DISCS[0];
  const candidateUrls = [
    ...getSoundUrls(disc.soundPath),
    ...(disc.altSoundPath ? getSoundUrls(disc.altSoundPath) : [])
  ];

  const discImg = document.getElementById('jukebox-disc-img');
  if (discImg) {
    attachMultiSourceImage(discImg, getTextureUrls(disc.tex));
  }

  let urlIndex = 0;
  const tryNextStream = () => {
    if (!bgMusicAudio) return;
    if (urlIndex < candidateUrls.length) {
      bgMusicAudio.src = candidateUrls[urlIndex++];
      bgMusicAudio.volume = bgMusicVolume;
      if (autoPlay && !userPausedMusic && !dmPlaying) {
        bgMusicAudio.play().then(() => {
          bgMusicPlaying = true;
          updateJukeboxUI();
        }).catch((err) => {
          if (err && err.name === 'NotAllowedError') {
            bgMusicPlaying = false;
            updateJukeboxUI();
          } else {
            tryNextStream();
          }
        });
      }
    }
  };

  bgMusicAudio.onerror = tryNextStream;
  tryNextStream();
  updateJukeboxUI();
}

function updateJukeboxUI() {
  const disc = JUKEBOX_DISCS[currentDiscIndex] || JUKEBOX_DISCS[0];
  const nowPlayingEl = document.getElementById('jukebox-now-playing');
  const statusSubEl = document.getElementById('jukebox-status-sub');
  const toggleBtn = document.getElementById('jukebox-toggle-btn');
  const discImg = document.getElementById('jukebox-disc-img');

  const volPct = Math.round(bgMusicVolume * 100);
  if (nowPlayingEl) {
    nowPlayingEl.textContent = `${bgMusicPlaying ? 'Playing' : 'Paused'}: ${disc.shortName}`;
  }
  if (statusSubEl) {
    statusSubEl.textContent = `${disc.title} • Vol ${volPct}%`;
  }
  if (toggleBtn) {
    toggleBtn.textContent = bgMusicPlaying ? 'PAUSE' : 'PLAY';
  }
  if (discImg) {
    discImg.classList.toggle('disc-paused', !bgMusicPlaying);
  }
}

// ==============================================================================
// 7. Advancement Toasts, Tooltips, Releases, Console & Tribute
// ==============================================================================

function showAdvancementToast(title, subtitle, texPath = 'item/diamond.png') {
  const stack = document.getElementById('advancement-toast-stack');
  if (!stack) return;

  const toast = document.createElement('div');
  toast.className = 'mc-advancement-toast';

  const slot = document.createElement('div');
  slot.className = 'toast-icon-slot';
  const img = document.createElement('img');
  img.className = 'mc-png mc-png-lg';
  img.alt = title;
  attachMultiSourceImage(img, getTextureUrls(texPath));
  slot.appendChild(img);

  const textWrap = document.createElement('div');
  textWrap.innerHTML = `
    <div class="mc-e" style="font-size: 12.5px;">${title}</div>
    <div class="mc-f" style="font-size: 12px;">${subtitle}</div>
  `;

  toast.appendChild(slot);
  toast.appendChild(textWrap);
  stack.appendChild(toast);

  setTimeout(() => {
    toast.classList.add('hiding');
    setTimeout(() => {
      if (toast.parentNode) toast.parentNode.removeChild(toast);
    }, 340);
  }, 3600);
}

function initFloatingTooltips() {
  const tooltip = document.getElementById('mc-floating-tooltip');
  if (!tooltip) return;

  document.querySelectorAll('[data-tooltip-title]').forEach((el) => {
    el.addEventListener('mouseenter', () => {
      const title = el.getAttribute('data-tooltip-title') || '';
      const lore = (el.getAttribute('data-tooltip-lore') || '').split('|');
      tooltip.innerHTML = '';
      tooltip.appendChild(parseMinecraftColorText(title, 'mc-f'));
      lore.forEach((line) => {
        if (line.trim()) tooltip.appendChild(parseMinecraftColorText(line, 'mc-7'));
      });
      tooltip.style.display = 'block';
    });

    el.addEventListener('mousemove', (e) => {
      const x = Math.min(window.innerWidth - 320, e.clientX + 20);
      const y = Math.min(window.innerHeight - 110, e.clientY + 20);
      tooltip.style.left = `${Math.max(8, x)}px`;
      tooltip.style.top = `${Math.max(8, y)}px`;
    });

    el.addEventListener('mouseleave', () => {
      tooltip.style.display = 'none';
    });
  });
}

function initScrollReveal() {
  const items = document.querySelectorAll('.reveal-on-scroll');
  if (!('IntersectionObserver' in window)) {
    items.forEach((el) => el.classList.add('revealed'));
    return;
  }
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('revealed');
        }
      });
    },
    { threshold: 0.08 }
  );
  items.forEach((el) => observer.observe(el));
}

function formatBytes(bytes) {
  if (!bytes || isNaN(bytes)) return '';
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(2)} MB`;
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${bytes} B`;
}

async function initReleaseDownloader() {
  const syncStatusEl = document.getElementById('release-sync-status');
  const pubDateEl = document.getElementById('release-pub-date');
  const refreshBtn = document.getElementById('refresh-releases-btn');
  const copyBtn = document.getElementById('copy-install-btn');

  if (refreshBtn) {
    refreshBtn.addEventListener('click', () => {
      fetchAndBindReleases(true);
    });
  }

  if (copyBtn) {
    copyBtn.addEventListener('click', () => {
      const cmdText = document.getElementById('install-text')?.textContent || '';
      copyCommandToClipboard(cmdText);
    });
  }

  await fetchAndBindReleases(false);

  async function fetchAndBindReleases(showToastOnComplete) {
    if (syncStatusEl) {
      syncStatusEl.textContent = `Checking api.github.com/repos/${SAWBYTE_REPO}/releases ...`;
    }

    let defaultBranch = 'main';
    try {
      const repoRes = await fetch(`https://api.github.com/repos/${SAWBYTE_REPO}`);
      if (repoRes.ok) {
        const repoData = await repoRes.json();
        defaultBranch = repoData.default_branch || 'main';
        const starEl = document.getElementById('nav-star-count');
        if (starEl && typeof repoData.stargazers_count === 'number') {
          starEl.textContent = `★ ${repoData.stargazers_count}`;
        }
        document.querySelectorAll('[data-repo-link]').forEach((a) => {
          a.href = repoData.html_url || `https://github.com/${SAWBYTE_REPO}`;
        });
        document.querySelectorAll('[data-repo-releases-link]').forEach((a) => {
          a.href = `${repoData.html_url || `https://github.com/${SAWBYTE_REPO}`}/releases`;
        });
        document.querySelectorAll('[data-repo-actions-link]').forEach((a) => {
          a.href = `${repoData.html_url || `https://github.com/${SAWBYTE_REPO}`}/actions`;
        });
      }
    } catch (_) {}

    const installCmd = `curl -sSfL https://raw.githubusercontent.com/${SAWBYTE_REPO}/${defaultBranch}/packaging/install.sh | bash`;
    const installEl = document.getElementById('install-text');
    if (installEl) installEl.textContent = installCmd;

    const skillDocLink = document.getElementById('skill-doc-link');
    const skillRawLink = document.getElementById('skill-raw-link');
    const footerSkillLink = document.getElementById('footer-skill-link');
    const footerLicenseLink = document.getElementById('footer-license-link');
    if (skillDocLink) skillDocLink.href = `https://github.com/${SAWBYTE_REPO}/blob/${defaultBranch}/skills/sawbyte/SKILL.md`;
    if (skillRawLink) skillRawLink.href = `https://raw.githubusercontent.com/${SAWBYTE_REPO}/${defaultBranch}/skills/sawbyte/SKILL.md`;
    if (footerSkillLink) footerSkillLink.href = `https://github.com/${SAWBYTE_REPO}/blob/${defaultBranch}/skills/sawbyte/SKILL.md`;
    if (footerLicenseLink) footerLicenseLink.href = `https://github.com/${SAWBYTE_REPO}/blob/${defaultBranch}/LICENSE`;

    let release = null;
    try {
      const latestRes = await fetch(`https://api.github.com/repos/${SAWBYTE_REPO}/releases/latest`);
      if (latestRes.ok) {
        release = await latestRes.json();
      } else {
        const listRes = await fetch(`https://api.github.com/repos/${SAWBYTE_REPO}/releases?per_page=5`);
        if (listRes.ok) {
          const list = await listRes.json();
          if (Array.isArray(list) && list.length > 0) release = list[0];
        }
      }
    } catch (_) {}

    if (release && Array.isArray(release.assets)) {
      const tag = release.tag_name || 'latest';
      document.querySelectorAll('.release-tag-badge').forEach((el) => {
        el.textContent = tag;
      });

      if (pubDateEl && release.published_at) {
        const d = new Date(release.published_at);
        pubDateEl.textContent = d.toISOString().split('T')[0];
      }
      if (syncStatusEl) {
        syncStatusEl.innerHTML = `Synced with <span class="mc-f">${SAWBYTE_REPO} (${tag})</span> • ${release.assets.length} files`;
      }

      bindReleaseAssetsToButtons(release.assets, tag);
      renderLiveAssetsTable(release.assets, release.html_url);

      if (showToastOnComplete) {
        showAdvancementToast('Releases Synced', `Loaded ${release.assets.length} files from ${tag}`, 'item/emerald.png');
      }
    } else {
      const dynamicLatestBase = `https://github.com/${SAWBYTE_REPO}/releases/latest/download`;
      document.querySelectorAll('.release-tag-badge').forEach((el) => {
        el.textContent = 'Latest';
      });
      if (syncStatusEl) {
        syncStatusEl.textContent = `Connected to GitHub latest release redirect (${SAWBYTE_REPO})`;
      }
      if (pubDateEl) pubDateEl.textContent = 'Auto Redirect';

      const fallbackAssets = [
        { name: 'sawbyte-launcher-ubuntu-24.04-X64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-launcher-ubuntu-24.04-X64.tar.gz` },
        { name: 'sawbyte-ubuntu-24.04-X64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-ubuntu-24.04-X64.tar.gz` },
        { name: 'sawbyte-play-linux-x64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-play-linux-x64.tar.gz` },
        { name: 'sawbyte-launcher-windows-2022-X64.zip', browser_download_url: `${dynamicLatestBase}/sawbyte-launcher-windows-2022-X64.zip` },
        { name: 'sawbyte-windows-2022-X64.zip', browser_download_url: `${dynamicLatestBase}/sawbyte-windows-2022-X64.zip` },
        { name: 'sawbyte-play-windows-x64.zip', browser_download_url: `${dynamicLatestBase}/sawbyte-play-windows-x64.zip` },
        { name: 'sawbyte-launcher-macos-14-ARM64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-launcher-macos-14-ARM64.tar.gz` },
        { name: 'sawbyte-macos-14-ARM64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-macos-14-ARM64.tar.gz` },
        { name: 'sawbyte-play-macos-arm64.tar.gz', browser_download_url: `${dynamicLatestBase}/sawbyte-play-macos-arm64.tar.gz` }
      ];
      bindReleaseAssetsToButtons(fallbackAssets, 'Latest');
      renderLiveAssetsTable(fallbackAssets, `https://github.com/${SAWBYTE_REPO}/releases/latest`);
    }
  }
}

function bindReleaseAssetsToButtons(assets, tag) {
  const matchers = {
    'linux-launcher': (n) => n.includes('launcher') && (n.includes('ubuntu') || n.includes('linux')),
    'linux-engine': (n) => n.includes('play') && (n.includes('linux') || n.includes('ubuntu')),
    'linux-play': (n) => !n.includes('launcher') && !n.includes('play') && (n.includes('ubuntu') || n.includes('linux')),
    'win-launcher': (n) => n.includes('launcher') && n.includes('win'),
    'win-engine': (n) => n.includes('play') && n.includes('win'),
    'win-play': (n) => !n.includes('launcher') && !n.includes('play') && n.includes('win'),
    'mac-launcher': (n) => n.includes('launcher') && (n.includes('mac') || n.includes('darwin')),
    'mac-engine': (n) => n.includes('play') && (n.includes('mac') || n.includes('darwin')),
    'mac-play': (n) => !n.includes('launcher') && !n.includes('play') && (n.includes('mac') || n.includes('darwin')),
    'bridge-jar': (n) => n.includes('bridge') && n.endsWith('.jar')
  };

  for (const [key, matchFn] of Object.entries(matchers)) {
    const btn = document.querySelector(`[data-dl-key="${key}"]`);
    if (!btn) continue;
    const found = assets.find((a) => matchFn((a.name || '').toLowerCase()));
    if (found) {
      btn.href = found.browser_download_url;
      const badge = btn.querySelector('.dl-meta-badge');
      const ext = found.name.endsWith('.tar.gz') ? '.tar.gz' : `.${found.name.split('.').pop()}`;
      const sizeText = formatBytes(found.size);
      if (badge) {
        badge.textContent = sizeText ? `${sizeText} (${ext})` : ext;
      }
      if (key === 'bridge-jar') {
        const bridgeLabel = document.getElementById('bridge-jar-btn-label');
        if (bridgeLabel) {
          bridgeLabel.textContent = `Download ${found.name}${sizeText ? ` (${sizeText})` : ''}`;
        }
      }
    } else {
      btn.href = `https://github.com/${SAWBYTE_REPO}/releases`;
      const badge = btn.querySelector('.dl-meta-badge');
      if (badge) badge.textContent = tag;
      if (key === 'bridge-jar') {
        const bridgeLabel = document.getElementById('bridge-jar-btn-label');
        if (bridgeLabel) bridgeLabel.textContent = `Open Releases (${tag})`;
      }
    }
  }
}

function renderLiveAssetsTable(assets, fallbackUrl) {
  const tbody = document.getElementById('live-assets-tbody');
  if (!tbody) return;
  tbody.innerHTML = '';

  assets.forEach((asset) => {
    const tr = document.createElement('tr');
    const name = asset.name || 'binary';
    const lower = name.toLowerCase();
    let platform = 'Cross-Platform';
    if (lower.includes('ubuntu') || lower.includes('linux')) platform = 'Linux x86_64';
    else if (lower.includes('win')) platform = 'Windows x64';
    else if (lower.includes('mac')) platform = 'macOS ARM64';
    else if (lower.includes('bridge') || lower.endsWith('.jar')) platform = `Fabric ${TARGET_MC_VERSION} Mod`;

    const sizeStr = formatBytes(asset.size) || 'Direct Link';
    const dlCount = typeof asset.download_count === 'number' ? asset.download_count.toLocaleString() : 'Live';
    const url = asset.browser_download_url || fallbackUrl;

    tr.innerHTML = `
      <td class="mc-b">${name}</td>
      <td class="mc-7">${platform}</td>
      <td class="mc-e">${sizeStr}</td>
      <td class="mc-a">${dlCount}</td>
      <td><a href="${url}" target="_blank" rel="noopener" class="status-tag" style="text-decoration: none;">Download -&gt;</a></td>
    `;
    tbody.appendChild(tr);
  });
}

function copyCommandToClipboard(text) {
  const label = document.getElementById('copy-install-label');
  const finishCopy = () => {
    playMcXpChime();
    if (label) {
      label.textContent = 'COPIED!';
      setTimeout(() => {
        label.textContent = 'COPY';
      }, 2000);
    }
    showAdvancementToast('Command Copied!', 'Paste it into your terminal to install', 'block/command_block_front.png');
  };

  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(text).then(finishCopy).catch(() => fallbackCopy(text, finishCopy));
  } else {
    fallbackCopy(text, finishCopy);
  }
}

function fallbackCopy(text, cb) {
  const ta = document.createElement('textarea');
  ta.value = text;
  document.body.appendChild(ta);
  ta.select();
  try {
    document.execCommand('copy');
  } catch (_) {}
  document.body.removeChild(ta);
  if (cb) cb();
}

const MC_CODE_MAP = {
  '0': 'mc-0', '1': 'mc-1', '2': 'mc-2', '3': 'mc-3',
  '4': 'mc-4', '5': 'mc-5', '6': 'mc-6', '7': 'mc-7',
  '8': 'mc-8', '9': 'mc-9', 'a': 'mc-a', 'b': 'mc-b',
  'c': 'mc-c', 'd': 'mc-d', 'e': 'mc-e', 'f': 'mc-f'
};

function parseMinecraftColorText(rawText, defaultClass = 'mc-7') {
  const container = document.createElement('div');
  container.style.marginBottom = '4px';

  const parts = String(rawText).split(/(§[0-9a-frA-FR])/g);
  let currentClass = defaultClass;

  for (const part of parts) {
    if (part.startsWith('§') && part.length === 2) {
      const code = part[1].toLowerCase();
      if (code === 'r') currentClass = defaultClass;
      else if (MC_CODE_MAP[code]) currentClass = MC_CODE_MAP[code];
    } else if (part.length > 0) {
      const span = document.createElement('span');
      span.className = currentClass;
      span.textContent = part;
      container.appendChild(span);
    }
  }
  return container;
}

window.insertColorCode = function (code) {
  const input = document.getElementById('term-input');
  if (!input) return;
  input.value += code;
  input.focus();
  playMcClick(1.25);
};

window.runTerminalQuickCommand = function (cmd) {
  const input = document.getElementById('term-input');
  if (!input) return;
  input.value = cmd;
  const ev = new KeyboardEvent('keydown', { key: 'Enter' });
  input.dispatchEvent(ev);
};

function initTerminalConsole() {
  const input = document.getElementById('term-input');
  const body = document.getElementById('term-body');
  if (!input || !body) return;

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      const cmd = input.value.trim();
      input.value = '';
      if (!cmd) return;

      playMcClick(1.1);
      appendFormattedLine(`§a$ §f${cmd}`);
      handleCommand(cmd);
    }
  });

  function appendFormattedLine(text, defaultClass = 'mc-7') {
    const lineElem = parseMinecraftColorText(text, defaultClass);
    body.appendChild(lineElem);
    body.scrollTop = body.scrollHeight;
  }

  function handleCommand(cmd) {
    if (cmd.includes('§')) {
      appendFormattedLine(`§8[Preview] §r${cmd}`);
      return;
    }

    const parts = cmd.split(' ');
    const root = parts[0].toLowerCase();

    switch (root) {
      case 'help':
        appendFormattedLine('§6Sawbyte Console Commands:');
        appendFormattedLine('  §binspect.status                 §7: Show coordinates, health, and tick rate');
        appendFormattedLine('  §bdisc_maker                     §7: Generate & play a random stochastic track');
        appendFormattedLine('  §btool.swap                      §7: Roll a new random cursor weapon/tool');
        appendFormattedLine('  §bsound.pigstep                  §7: Play Lena Raine Pigstep disc');
        appendFormattedLine('  §btechnoblade                    §7: Pay respects [o7]');
        appendFormattedLine('  §bcolors                         §7: Show all §0 to §f color codes');
        appendFormattedLine('  §bclear                          §7: Clear console');
        break;
      case 'colors':
        appendFormattedLine('§0§0Black §1§1DarkBlue §2§2DarkGreen §3§3DarkAqua §4§4DarkRed §5§5Purple §6§6Gold §7§7Gray');
        appendFormattedLine('§8§8DarkGray §9§9Blue §a§aGreen §b§bAqua §c§cRed §d§dLightPurple §e§eYellow §f§fWhite');
        break;
      case 'inspect.status':
        appendFormattedLine(`§apos=(8.50, 64.00, 8.50) §emc=${TARGET_MC_VERSION} §btool="${currentTool.name}" §amined=${totalMinedBlocks} §btps=20.00`);
        break;
      case 'disc_maker':
        if (typeof window.startDiscMakerFromConsole === 'function') {
          window.startDiscMakerFromConsole();
        }
        appendFormattedLine('§d[disc_maker] §fRandomized Euclidean + Markov Minecraft beat & started studio!');
        break;
      case 'tool.swap':
        if (typeof window.swapCursorToolNow === 'function') {
          window.swapCursorToolNow();
        }
        appendFormattedLine(`§b[Cursor Tool] §fEquipped §e${currentTool.name} §7(cooldown ${(currentTool.cooldownMs / 1000).toFixed(2)}s)`);
        break;
      case 'sound.pigstep':
        if (dmPlaying) stopDiscMaker();
        currentDiscIndex = 0;
        userPausedMusic = false;
        loadDiscIntoPlayer(0, true);
        appendFormattedLine('§d[Jukebox] §fPlaying §6Lena Raine: Pigstep');
        break;
      case 'technoblade':
        playMcTotemSound();
        appendFormattedLine('§6[Crown] §e“Technoblade Never Dies!” §c[o7]');
        break;
      case 'clear':
        body.innerHTML = '';
        break;
      default:
        appendFormattedLine(`§cUnknown command '${cmd}'. Type §ehelp §cfor a list of commands.`);
    }
  }
}

function initBenchmarkTabs() {
  const tabs = document.querySelectorAll('.bench-tab-btn');
  const sections = document.querySelectorAll('.bench-content-section');
  tabs.forEach((tab) => {
    tab.addEventListener('click', () => {
      tabs.forEach((t) => t.classList.remove('active'));
      sections.forEach((s) => (s.style.display = 'none'));
      tab.classList.add('active');
      const target = document.getElementById(tab.dataset.target);
      if (target) target.style.display = 'block';
    });
  });
}

let potatoWarTotal = 500000000;

window.triggerTechnoSound = function (type) {
  switch (type) {
    case 'pig':
      playMcPigSound();
      showAdvancementToast('Oink! [o7]', 'Played Minecraft Pig sound (mob/pig/say.ogg)', 'item/porkchop.png');
      break;
    case 'totem':
      playMcTotemSound();
      showAdvancementToast('Technoblade Never Dies!', 'Activated Totem of Undying', 'item/totem_of_undying.png');
      break;
    case 'potato':
      potatoWarTotal += Math.floor(Math.random() * 500) + 100;
      const formatted = potatoWarTotal.toLocaleString('en-US');
      const p1 = document.getElementById('potato-war-count');
      const p2 = document.getElementById('potato-badge-count');
      if (p1) p1.textContent = formatted;
      if (p2) p2.textContent = formatted;
      playMcEatSound();
      setTimeout(() => playMcXpChime(), 180);
      showAdvancementToast('Skyblock Potato War #1', `Farmed ${formatted} Baked Potatoes!`, 'item/baked_potato.png');
      break;
    case 'bell':
      playMcBellSound();
      showAdvancementToast('Channel Member Bell!', '“One of Us!”', 'item/bell.png');
      break;
    case 'pigstep':
      if (dmPlaying) stopDiscMaker();
      currentDiscIndex = 0;
      userPausedMusic = false;
      loadDiscIntoPlayer(0, true);
      showAdvancementToast('Playing: Pigstep', 'Lena Raine • Nether Update Disc', 'item/music_disc_pigstep.png');
      break;
    case 'firework':
      playMcFireworkSound();
      showAdvancementToast('Firework Salute', 'Launched tribute firework [o7]', 'item/firework_rocket.png');
      break;
  }
};

function initRespectsAndTechnoSoundboard() {
  const btn = document.getElementById('pay-respects-btn');
  const label = document.getElementById('respects-label');
  const countEl = document.getElementById('tribute-count');
  const cooldownBar = document.getElementById('tribute-cooldown-bar');
  const cooldownFill = document.getElementById('tribute-cooldown-fill');
  const statusHint = document.getElementById('tribute-status-hint');
  const wrap = document.querySelector('.tribute-actions-wrap');

  if (!btn || !label) return;

  const STORAGE_KEY_COUNT = 'sawbyte_techno_tributes';
  const STORAGE_KEY_LAST_TIME = 'sawbyte_techno_last_tribute_ts';
  const COOLDOWN_MS = 3500;

  function safeStorageGet(key, fallback) {
    try {
      const v = localStorage.getItem(key);
      return v !== null ? v : fallback;
    } catch (_) {
      return fallback;
    }
  }

  function safeStorageSet(key, value) {
    try {
      localStorage.setItem(key, String(value));
    } catch (_) {}
  }

  let tributeCount = parseInt(safeStorageGet(STORAGE_KEY_COUNT, '0'), 10) || 0;
  let lastTributeTime = parseInt(safeStorageGet(STORAGE_KEY_LAST_TIME, '0'), 10) || 0;
  let cooldownTimer = null;
  let isCooldown = false;

  function updateDisplay() {
    if (countEl) countEl.textContent = Number(tributeCount).toLocaleString('en-US');
    if (tributeCount > 0) {
      label.innerHTML = `<span class="mc-e">Respects Paid (o7 × ${tributeCount}) • Technoblade Never Dies</span>`;
    } else {
      label.textContent = 'Leave a Tribute Poppy [o7]';
    }
  }

  function spawnFloatingPoppy() {
    if (!wrap) return;
    const poppy = document.createElement('div');
    poppy.className = 'tribute-floating-poppy';

    const pImg = document.createElement('img');
    pImg.className = 'mc-png mc-png-sm';
    pImg.alt = 'Poppy';
    attachMultiSourceImage(pImg, getTextureUrls('block/poppy.png'));

    const textSpan = document.createElement('span');
    textSpan.textContent = '+1 Tribute Poppy [o7]';

    poppy.appendChild(pImg);
    poppy.appendChild(textSpan);

    const offset = (Math.random() - 0.5) * 90;
    poppy.style.transform = `translateX(${offset}px)`;
    poppy.style.top = '8px';

    wrap.appendChild(poppy);
    setTimeout(() => {
      if (poppy.parentNode) poppy.parentNode.removeChild(poppy);
    }, 1700);
  }

  function startCooldown(remainingMs) {
    isCooldown = true;
    btn.classList.add('tribute-btn-disabled');
    btn.setAttribute('aria-disabled', 'true');
    if (cooldownBar) cooldownBar.style.display = 'block';

    const startTime = Date.now() - (COOLDOWN_MS - remainingMs);
    if (cooldownTimer) clearInterval(cooldownTimer);

    cooldownTimer = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const left = Math.max(0, COOLDOWN_MS - elapsed);
      const pct = Math.min(100, (elapsed / COOLDOWN_MS) * 100);

      if (cooldownFill) cooldownFill.style.width = pct + '%';

      const secsLeft = Math.ceil(left / 1000);
      label.innerHTML = `<span class="mc-8">[Wait ${secsLeft}s]</span> <span class="mc-7">• Technoblade Never Dies</span>`;

      if (left <= 0) {
        clearInterval(cooldownTimer);
        cooldownTimer = null;
        isCooldown = false;
        btn.classList.remove('tribute-btn-disabled');
        btn.removeAttribute('aria-disabled');
        if (cooldownBar) cooldownBar.style.display = 'none';
        if (cooldownFill) cooldownFill.style.width = '0%';
        if (statusHint) statusHint.textContent = 'Plays Minecraft Pig & Level-Up sounds • Saved in browser storage';
        updateDisplay();
      }
    }, 80);
  }

  updateDisplay();
  const elapsedSinceLast = Date.now() - lastTributeTime;
  if (lastTributeTime > 0 && elapsedSinceLast < COOLDOWN_MS) {
    startCooldown(COOLDOWN_MS - elapsedSinceLast);
  }

  btn.addEventListener('click', (e) => {
    e.preventDefault();
    const now = Date.now();
    const elapsed = now - lastTributeTime;

    if (isCooldown || (lastTributeTime > 0 && elapsed < COOLDOWN_MS)) {
      playMcClick(0.65);
      return;
    }

    tributeCount++;
    lastTributeTime = now;
    safeStorageSet(STORAGE_KEY_COUNT, tributeCount);
    safeStorageSet(STORAGE_KEY_LAST_TIME, lastTributeTime);

    playMcLevelUp();
    setTimeout(() => playMcPigSound(), 280);

    spawnFloatingPoppy();
    updateDisplay();
    showAdvancementToast('Respects Paid [o7]', `Tribute #${tributeCount} • Technoblade Never Dies`, 'block/poppy.png');
    startCooldown(COOLDOWN_MS);
  });
}


// ==============================================================================
// 8. Site Experience Mode Selector (Minimal Default vs Interactive 2.5D World)
// ==============================================================================

window.setSiteExperienceMode = function (mode) {
  const cleanMode = mode === 'interactive' ? 'interactive' : 'minimal';
  document.body.setAttribute('data-site-mode', cleanMode);
  const btnMin = document.getElementById('mode-btn-minimal');
  const btnInt = document.getElementById('mode-btn-interactive');
  if (btnMin) btnMin.classList.toggle('active', cleanMode === 'minimal');
  if (btnInt) btnInt.classList.toggle('active', cleanMode === 'interactive');

  const studioBox = document.getElementById('disc-maker-studio-box');
  if (studioBox) {
    studioBox.classList.toggle('studio-expanded', cleanMode === 'interactive');
  }

  if (cleanMode === 'interactive') {
    if (typeof window.markWorldTerrainDirty === 'function') {
      window.markWorldTerrainDirty();
    }
    showAdvancementToast('Interactive 2.5D Mode', 'Animated Minecraft world & breakable blocks enabled!', 'block/grass_block_side.png');
  } else {
    showAdvancementToast('Minimal Clean Mode', 'Distraction-free engineering layout active', 'item/diamond.png');
  }
  playMcClick(1.1);
};

window.toggleCursorMode = function () {
  const cur = document.body.getAttribute('data-cursor-mode') === 'mc' ? 'normal' : 'mc';
  document.body.setAttribute('data-cursor-mode', cur);
  const btn = document.getElementById('cursor-mode-toggle-btn');
  if (btn) {
    btn.textContent = cur === 'mc' ? `Cursor: ${currentTool.name}` : 'Cursor: Standard';
    btn.classList.toggle('active', cur === 'mc');
  }
  playMcClick(1.15);
};

window.toggleDiscMakerDrawer = function () {
  const box = document.getElementById('disc-maker-studio-box');
  if (!box) return;
  box.classList.toggle('studio-expanded');
  playMcClick(1.05);
};
