import { useEffect, useRef, useState, type RefObject } from 'react';
import './App.css';
import music_video_svg from './assets/music_video_128dp_E3E3E3_FILL0_wght400_GRAD0_opsz48.svg';
import pause_svg from './assets/pause_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import play_svg from './assets/play_arrow_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import play_next_svg from './assets/skip_next_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg';
import axios from 'axios';

function App() {
    let played = 0;
    const audio_ref = useRef<HTMLAudioElement>(null);;
    const [title, _setTitle] = useState("Audio title");
    const [position, setPosition] = useState(0.0);
    const [duration, setDuration] = useState(1.0);
    const [is_playing, setIsPlaying] = useState(false);

    useEffect(() => {
        navigator.mediaSession.setActionHandler('nexttrack', () => {
            onPlayNextClick(audio_ref, played++);
        })
    }, []);

    useEffect(() => {
        fetchRandomAudioFile(played).then(href => {
            if (audio_ref.current === null) return
            audio_ref.current.src = href
        })
    }, [audio_ref]);

    useEffect(() => {
        if (is_playing) {
            audio_ref.current?.play();
        } else {
            audio_ref.current?.pause();
        }
    }, [is_playing, audio_ref]);
    useEffect(() => {
        if (audio_ref.current !== null) {
            audio_ref.current.onloadedmetadata = () => {
                setDuration(audio_ref.current!.duration);
            }
            audio_ref.current.ontimeupdate = () => {
                setPosition(Math.ceil(audio_ref.current!.currentTime));
            }
            audio_ref.current.onended = () => {
                played++;
                fetchRandomAudioFile(played).then(href => {
                    if (audio_ref.current === null) return;
                    window.URL.revokeObjectURL(audio_ref.current.src);
                    audio_ref.current.src = href;
                    audio_ref.current.play();
                })
            }
        }
    }, [audio_ref]);


    return (
        <>
            <audio className="hidden" controls ref={audio_ref}></audio>
            <h1 className="p-4">SubSonicVault Player</h1>
            <div className="card bg-base-100 w-96 shadow-sm mx-auto">
                <figure>
                    <img src={music_video_svg} alt='music video icon' width="256" />
                </figure>
                <div className="card-body">
                    <h2 className="card-title mx-auto">{title}</h2>
                    <input type="range" min="0.0" value={position} max={duration} className="range range-xs w-full" onChange={(e) => { setPosition(+e.target.value) }} />
                    <div className="flex w-full">
                        <p className="text-left">{toHHMMSS(position)}</p>
                        <p className="text-right">{toHHMMSS(Math.ceil(duration))}</p>
                    </div>
                    <div className="relative flex items-center justify-center gap-1">
                        <div className="w-10" />
                        <button className="btn btn-primary btn-xl btn-circle" onClick={() => setIsPlaying((is_playing) => !is_playing)}>
                            <PlayPauseButtonIcon is_playing={is_playing} />
                        </button>
                        <button className="btn btn-primary btn-l btn-circle" onClick={() => onPlayNextClick(audio_ref, played++)}>
                            <img src={play_next_svg} alt='play next icon' width="38" />
                        </button>
                    </div>
                </div>
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

async function fetchRandomAudioFile(played: number): Promise<string> {
    const response = await axios({
        method: 'get',
        url: `/?${played}`,
        responseType: 'blob'
    });

    const href = window.URL.createObjectURL(response.data);

    return href;
}

async function onPlayNextClick(audio_ref: RefObject<HTMLAudioElement | null>, played: number) {
    fetchRandomAudioFile(played).then(href => {
        if (audio_ref.current === null) return;
        window.URL.revokeObjectURL(audio_ref.current.src);
        audio_ref.current.src = href;
        audio_ref.current.play();
    });
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
