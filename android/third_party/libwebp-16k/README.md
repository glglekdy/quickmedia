# libwebp (16KB 페이지용)

youtubedl-android ffmpeg 0.18.1 의 `libffmpeg.zip.so` 안에 든 libwebp 계열 5개는 4KB 정렬로 빌드돼 있어서,
16KB 페이지 기기(안드로이드 15+ 의 16KB 모드, `ps16k` 에뮬레이터)에서는 ffmpeg 가 아예 뜨지 않는다.
그러면 영상·음성 합치기, mp3 바꾸기, 구간 자르기가 모두 실패한다.
([yausername/youtubedl-android#365](https://github.com/yausername/youtubedl-android/issues/365))

그래서 같은 이름의 16KB 정렬 빌드를 앱에 넣고, 16KB 페이지 기기에서만 앱이 켜질 때 풀린 파일 중 4KB 정렬인 것을
바꿔 놓는다(`download/PageSizeFix.kt`). 라이브러리가 고쳐지면 바꿀 것이 없어 아무것도 하지 않는다.

## 출처

Termux 의 `libwebp 1.6.0.0.0.1+really1.6.0` 패키지에서 꺼냈다.

| 아키텍처 | 패키지 | SHA-256 |
| --- | --- | --- |
| arm64-v8a | `libwebp_1.6.0.0.0.1+really1.6.0_aarch64.deb` | `8ba7fcfadd1a9957a7632c6407d6ef3e41d34dbd96a41769b245592ed4559045` |
| x86_64 | `libwebp_1.6.0.0.0.1+really1.6.0_x86_64.deb` | `3274f3b6b3b765ed5cfb3d33cf309a2222de7766ad53f0eaefcbe89563f2f7a3` |

받은 곳: `https://packages.termux.dev/apt/termux-main/pool/main/libw/libwebp/`

원래 파일과 SONAME 이 같고, 원래 파일이 내보내는 심볼을 모두 내보낸다(심볼 버전 없음).
x86_64 에뮬레이터(16KB)에서 바꿔 넣은 뒤 ffmpeg 7.1.1 이 뜨고 영상·음성 합치기가 되는 것을 확인했다.

라이선스: BSD-3-Clause, `COPYING`.
