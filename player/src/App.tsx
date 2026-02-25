import { useEffect, useRef, useState } from 'react'
import './App.css'
import music_video_svg from './assets/music_video_128dp_E3E3E3_FILL0_wght400_GRAD0_opsz48.svg'
import pause_svg from './assets/pause_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg'
import play_svg from './assets/play_arrow_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.svg'

function App() {
    const audio_ref = useRef<HTMLAudioElement>(null);
    const [title, _setTitle] = useState("Audio title")
    const [position, setPosition] = useState(0.0)
    const [duration, setDuration] = useState(1.0)
    const [is_playing, setIsPlaying] = useState(false)

    useEffect(() => {
        if (is_playing) {
            audio_ref.current?.play()
        } else {
            audio_ref.current?.pause()
        }
    }, [is_playing, audio_ref])
    useEffect(() => {
        if (audio_ref.current !== null) {
            audio_ref.current.onloadeddata = () => {
                setDuration(audio_ref.current!.duration)
            }
            audio_ref.current.ontimeupdate = () => {
                setPosition(Math.ceil(audio_ref.current!.currentTime))
            }
        }
    }, [audio_ref])


    return (
        <>
            <audio className="hidden" src="./" controls ref={audio_ref}></audio>
            <h1 className="p-4">SubSonicVault Player</h1>
            <div className="card bg-base-100 w-96 shadow-sm mx-auto">
                <figure>
                    <img src={music_video_svg} alt='music video icon' width="256" />
                </figure>
                <div className="card-body">
                    <h2 className="card-title mx-auto">{title}</h2>
                    <input type="range" min="0.0" value={position} max={duration} className="range range-xs" onChange={(e) => { setPosition(+e.target.value) }} />
                    <div className="flex">
                        <p className="text-left">{position}</p>
                        <p className="text-right">{Math.ceil(duration)}</p>
                    </div>
                    <button className="btn btn-primary btn-xl btn-circle mx-auto" onClick={() => setIsPlaying((is_playing) => !is_playing)}>
                        <PlayPauseButtonIcon is_playing={is_playing} />
                    </button>
                </div>
            </div>
        </>
    )
}

function PlayPauseButtonIcon({ is_playing }: { is_playing: boolean }) {
    if (is_playing) {
        return <img src={pause_svg} alt='pause icon' width="48" />
    } else {
        return <img src={play_svg} alt='play icon' width="48" />
    }
}
export default App
