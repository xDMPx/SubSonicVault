export interface AudioFile {
    id: string,
    path: string,
    mime: string,
}

export interface AudioFileBlob {
    href: string,
    id: string
}

export interface AudioFileMetadata {
    title: string | null,
    artist: string | null,
    album: string | null,
    genre: string | null,
    release_year: string | null,
    artwork_url: string | null,
    duration: number,
}
