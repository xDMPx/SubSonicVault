import { useEffect, useRef, useState, type RefObject } from 'react';
import './App.css';
import music_video_svg from './assets/music_video_128dp_E3E3E3_FILL0_wght400_GRAD0_opsz48.svg';
import pause_svg from './assets/pause_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import play_svg from './assets/play_arrow_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import play_next_svg from './assets/skip_next_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import play_prev_svg from './assets/skip_previous_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import volume_up_svg from './assets/volume_up_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import volume_off_svg from './assets/volume_off_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import axios, { type AxiosResponse } from 'axios';

import mediaInfoFactory from 'mediainfo.js';
import type { MediaInfo } from 'mediainfo.js';
import mediaInfoWasmUrl from 'mediainfo.js/MediaInfoModule.wasm?url';

function App() {
    const fetch_audio_files = useRef(true);
    const load_audio = useRef(true);

    const history = useRef([] as string[]);
    const current_his_index = useRef(0);

    const audio_ref = useRef<HTMLAudioElement>(null);
    const mediaInfoRef = useRef<MediaInfo<'object'> | null>(null);
    const [title, setTitle] = useState("Audio title");
    const [performer, setPerformer] = useState("Audio performer");
    const [position, setPosition] = useState(0.0);
    const [duration, setDuration] = useState(1.0);
    const [is_playing, setIsPlaying] = useState(false);
    const [is_muted, setIsMuted] = useState(false);
    const [playback_volume, setPlaybackVolume] = useState(1.0);
    const [audio_files, setAudioFiles] = useState([] as AudioFile[]);

    useEffect(() => {
        if (!fetch_audio_files.current) return;
        fetch_audio_files.current = false;
        fetchAudioFiles().then((audio_files) => {
            setAudioFiles(audio_files);
        });
    }, []);

    useEffect(() => {
        if (audio_files.length == 0) return;
        if (audio_ref.current === null) return;
        if (load_audio.current) {
            load_audio.current = false;
            fetchRandomAudioFile(audio_files).then(({ id, href }) => {
                if (audio_ref.current === null) return;
                history.current.push(id);
                audio_ref.current.src = href;
            });
        }
        audio_ref.current.onended = () => {
            if (audio_ref.current === null) return;
            window.URL.revokeObjectURL(audio_ref.current.src);
            current_his_index.current++;
            if (current_his_index.current === history.current.length) {
                fetchRandomAudioFile(audio_files).then(({ id, href }) => {
                    if (audio_ref.current === null) return;
                    history.current.push(id);
                    audio_ref.current.src = href;
                    audio_ref.current.play();
                });
            } else {
                const audio_id = history.current[current_his_index.current];
                fetchAudioFileById(audio_id).then(({ href }) => {
                    if (audio_ref.current === null) return;
                    audio_ref.current.src = href;
                    audio_ref.current.play();
                });
            }
        }
    }, [audio_files, audio_ref]);

    useEffect(() => {
        mediaInfoFactory({
            format: 'object',
            locateFile: (path, prefix) =>
                path === 'MediaInfoModule.wasm' ? mediaInfoWasmUrl : `${prefix}${path}`,
        }).then((mi) => {
            mediaInfoRef.current = mi;
        }).catch((error: unknown) => {
            console.error("mediaInfoFactory Error");
            console.error(error);
        });

        return () => {
            if (mediaInfoRef.current) {
                mediaInfoRef.current.close()
            }
        };
    }, []);

    useEffect(() => {
        navigator.mediaSession.setActionHandler('previoustrack', () => {
            onPlayPrevClick(audio_ref, history, current_his_index);
        });
        navigator.mediaSession.setActionHandler('nexttrack', () => {
            onPlayNextClick(audio_ref, audio_files, history, current_his_index);
        });
        navigator.mediaSession.setActionHandler('play', () => {
            audio_ref.current?.play();
            setIsPlaying(true);
        });
        navigator.mediaSession.setActionHandler('pause', () => {
            audio_ref.current?.pause();
            setIsPlaying(false);
        });
    }, [audio_files]);

    useEffect(() => {
        if (audio_ref.current === null) return;

        audio_ref.current.onplay = () => {
            setIsPlaying(true);
        }
        audio_ref.current.onpause = () => {
            setIsPlaying(false);
        }

        audio_ref.current.onloadstart = () => {
            const id = history.current[current_his_index.current];
            fetchAudioFileMetadata(id).then(metadata => {
                const duration = metadata.duration;
                setDuration(duration);
                const title = metadata.title;
                if (title === null) return;
                setTitle(title);

                const performer = metadata.artist;
                if (performer === null) return;
                setPerformer(performer);

                navigator.mediaSession.metadata = new MediaMetadata({
                    title: title,
                    artist: performer,
                });
            });
        }

        audio_ref.current.ontimeupdate = () => {
            setPosition(Math.ceil(audio_ref.current!.currentTime));
        }
    }, [audio_ref]);

    useEffect(() => {
        if (audio_ref.current === null) return;
        if (is_muted) {
            audio_ref.current.muted = true;
        } else {
            audio_ref.current.muted = false;
        }
    }, [is_muted]);


    useEffect(() => {
        if (audio_ref.current === null) return;
        audio_ref.current.volume = playback_volume;
    }, [playback_volume]);

    function seekToPosition(pos: number) {
        if (audio_ref.current === null) return;
        setPosition(pos);
        audio_ref.current.currentTime = pos;
    }

    function onPlayPauseClick() {
        if (is_playing) {
            audio_ref.current?.pause();
        } else {
            audio_ref.current?.play();
        }
    }

    function setPlayerVolume(playback_volume: number) {
        setPlaybackVolume(playback_volume / 100.0);
        if (playback_volume === 0.0) {
            setIsMuted(true);
        } else {
            setIsMuted(false);
        }
    }

    return (
        <>
            <audio className="hidden" controls ref={audio_ref}></audio>
            <div className="flex flex-col h-screen bg-base-200/30">
                <div className="navbar shadow-md">
                    <div className="navbar-start"></div>
                    <div className="navbar-center">
                        <h1 className="p-4">SubSonicVault Player</h1>
                    </div>
                    <div className="navbar-end"></div>
                </div>
                <div className="flex flex-1 justify-center place-items-center">
                    <div className="card w-full h-full sm:h-min sm:w-96 bg-base-100 shadow-lg">
                        <div className="my-auto">
                            <figure>
                                <img src={music_video_svg} alt='music video icon' width="256" />
                            </figure>
                            <div className="card-body">
                                <h2 className="card-title mx-auto">{title}</h2>
                                <h2 className="card-title mx-auto text-base">{performer}</h2>
                                <input type="range" min="0.0" value={position} max={duration} className="range range-xs w-full" onChange={(e) => {
                                    seekToPosition(+e.target.value)
                                }} />
                                <div className="flex w-full">
                                    <p className="text-left">{toHHMMSS(position)}</p>
                                    <p className="text-right">{toHHMMSS(Math.ceil(duration))}</p>
                                </div>
                                <div className="relative flex items-center justify-center gap-1">
                                    <button className="btn btn-primary btn-l btn-circle" onClick={() => onPlayPrevClick(audio_ref, history, current_his_index)}>
                                        <img src={play_prev_svg} alt='play previous' width="38" />
                                    </button>
                                    <button className="btn btn-primary btn-xl btn-circle" onClick={() => onPlayPauseClick()}>
                                        <PlayPauseButtonIcon is_playing={is_playing} />
                                    </button>
                                    <button className="btn btn-primary btn-l btn-circle" onClick={() => onPlayNextClick(audio_ref, audio_files, history, current_his_index)}>
                                        <img src={play_next_svg} alt='play next' width="38" />
                                    </button>
                                </div>
                                <div className="group items-center relative inline-flex w-min">
                                    <button className="btn btn-circle btn-ghost" onClick={() => setIsMuted(!is_muted)}>
                                        <VolumeButtonIcon is_muted={is_muted} />
                                    </button>
                                    <div className="w-0 overflow-hidden opacity-0 group-hover:w-32 group-hover:opacity-100">
                                        <input type="range" min="0" value={playback_volume * 100} max="100" className="range range-xs range-primary w-full" onChange={(e) => setPlayerVolume(+e.target.value)} />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div >
            </div>
        </>
    )
}

function PlayPauseButtonIcon({ is_playing }: { is_playing: boolean }) {
    if (is_playing) {
        return <img src={pause_svg} alt='pause icon' width="48" />;
    } else {
        return <img src={play_svg} alt='play icon' width="48" />;
    }
}

function VolumeButtonIcon({ is_muted }: { is_muted: boolean }) {
    if (is_muted) {
        return <img src={volume_off_svg} alt='pause icon' width="30" />;
    } else {
        return <img src={volume_up_svg} alt='play icon' width="30" />;
    }
}

interface AudioFile {
    id: string,
    path: string,
    mime: string,
}

async function fetchAudioFiles(): Promise<AudioFile[]> {
    let url = '/files';
    if (import.meta.env.DEV) {
        url = `http://localhost:65421${url}`;
    }
    const response: AxiosResponse<AudioFile[]> = await axios({
        method: 'get',
        url,
        responseType: 'json'
    });

    return response.data;
}

interface AudioFileBlob {
    href: string,
    id: string
}

async function fetchRandomAudioFile(audio_files: AudioFile[]): Promise<AudioFileBlob> {
    const id = audio_files.at(Math.floor(Math.random() * audio_files.length))?.id!;
    return fetchAudioFileById(id);
}

async function fetchAudioFileById(id: string): Promise<AudioFileBlob> {
    let url = `/file/${id}`;
    if (import.meta.env.DEV) {
        url = `http://localhost:65421${url}`;
    }
    const response = await axios({
        method: 'get',
        url,
        responseType: 'blob'
    });

    const href = window.URL.createObjectURL(response.data);

    return { href, id };
}

interface AudioFileMetadata {
    title: string | null,
    artist: string | null,
    album: string | null,
    genre: string | null,
    release_year: string | null,
    artwork_url: string | null,
    duration: number,
}

async function fetchAudioFileMetadata(id: string): Promise<AudioFileMetadata> {
    let url = `/file/${id}/metadata`;
    if (import.meta.env.DEV) {
        url = `http://localhost:65421${url}`;
    }
    const response = await axios({
        method: 'get',
        url,
        responseType: 'json'
    });



    return response.data;
}

async function onPlayNextClick(
    audio_ref: RefObject<HTMLAudioElement | null>, audio_files: AudioFile[],
    history: RefObject<string[]>, current_his_index: RefObject<number>) {
    if (audio_ref.current === null) return;
    window.URL.revokeObjectURL(audio_ref.current.src);
    current_his_index.current++;
    if (current_his_index.current === history.current.length) {
        fetchRandomAudioFile(audio_files).then(({ id, href }) => {
            if (audio_ref.current === null) return;
            history.current.push(id);
            audio_ref.current.src = href;
            audio_ref.current.play();
        });
    } else {
        const audio_id = history.current[current_his_index.current];
        fetchAudioFileById(audio_id).then(({ href }) => {
            if (audio_ref.current === null) return;
            audio_ref.current.src = href;
            audio_ref.current.play();
        });
    }
}

async function onPlayPrevClick(
    audio_ref: RefObject<HTMLAudioElement | null>,
    history: RefObject<string[]>, current_his_index: RefObject<number>) {
    current_his_index.current--;
    if (current_his_index.current < 0) {
        current_his_index.current = 0;
        if (audio_ref.current === null) return;
        audio_ref.current.currentTime = 0.0;
    } else {
        const audio_id = history.current[current_his_index.current];
        fetchAudioFileById(audio_id).then(({ href }) => {
            if (audio_ref.current === null) return;
            window.URL.revokeObjectURL(audio_ref.current.src);
            audio_ref.current.src = href;
            audio_ref.current.play();
        });
    }
}

function toHHMMSS(sec: number): string {
    if (sec < 0) return "00:00";

    const s = sec % 60;
    const m = Math.floor(sec % 3600 / 60);
    const h = Math.floor(sec / 3600);

    const ss = String(s).padStart(2, '0');
    const mm = String(m).padStart(2, '0');
    const hh = String(h).padStart(2, '0');

    if (h == 0) return `${mm}:${ss}`;
    return `${hh}:${mm}:${ss}`;
}

export default App;
