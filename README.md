<div align="center">
	<br>
	<p>
		<a href="https://github.com/sycertech/seiten"><img src="./.github/assets/logo.png" width="500" alt="seiten logo" /></a>
	</p>
	<br>
	<a href="https://discord.gg/RU3FhmX3Ja"><img alt="Discord Server" src="https://img.shields.io/discord/1041931589631881257?color=5865F2&logo=discord&logoColor=white"></a>
	<a href="https://github.com/sycertech/seiten/actions/workflows/ci.yml"><img alt="ci status" src="https://github.com/sycertech/seiten/actions/workflows/ci.yml/badge.svg"></a>
</div>

# About
**A simple deploy server for static sites**.

I've always used Vercel or Cloudflare Pages or GitHub Pages to serve my websites. But for a recent project, [crates.pm](https://github.com/sycertech/crates.pm), I wanted to run everything on one server -- my server. So, I opted to build our Next.js frontend to static files and serve them with nginx. But, I didn't want to have to manually upload the files every time I made a change. So, I built this simple server to handle that for me.  

Once configuring the server with a machine public GPG key, you can sign your archives in CD and post them to `/upload/:path`. The server will verify the signature and extract the archive to `/content/:path`. Then, you can configure nginx to serve the files from that directory.

## Usage
### Environment Variables
| Variable | Description | Default |
| --- | --- | --- |
| *`PUBLIC_GPG_KEY` | The public GPG key to use to verify signatures | None |
| `PORT` | The port to run the server on | `63208` |
| `DIRECTORIES` | A comma-separated list of directories to serve | Allow all |

## Example Docker Compose File
```yml
version: '3.7'

services:
  nginx:
    build:
      context: .
    restart: "unless-stopped"
    networks:
      - other_compose_service_networksa
      - other_compose_service_networksb
      - other_compose_service_networksc
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./conf.d:/etc/nginx/conf.d
      - ./index.html:/etc/nginx/html/index.html
      - ./content:/www
    ports:
      - 80:80
      - 443:443

  seiten:
    image: docker.io/sycertech/seiten:latest
    restart: "unless-stopped"
    volumes:
      - ./content:/content
    environment:
      PORT: 63208
      DIRECTORIES: example.com
      PUBLIC_GPG_KEY: |
        -----BEGIN PGP PUBLIC KEY BLOCK-----

        this is your key okay
        -----END PGP PUBLIC KEY BLOCK-----
    ports:
      - 80:63208
```

## GitHub Action
To make it easier to deploy your static sites, I've created a Github Action that will sign and upload your archives to the server.
```yml
- name: Upload to Seiten
  uses: sycertech/seiten/action@v1
  with:
    # the url of the seiten server (required)
    url: ${{ secrets.SEITEN_HOST }}
    # the private gpg key to sign the archive with (required)
    gpg-key: ${{ secrets.SEITEN_GPG_PRIVATE_KEY }}
    # the path to the archive to upload (required)
    archive: archive.tar.gz
    # the directory in /content to extract the archive to (required)
    path: crates.pm
```
You can see it in action in [crates.pm](https://github.com/sycertech/crates.pm/blob/682ef374a492285b8e838b51a4abc83b22908fa5/.github/workflows/web.yml).

## Words of warning
1. Make sure your archives are flat! The server will extract the archive to the specified path, so if your archive is not flat, it will extract to a subdirectory (read as: make sure index.html is at the root of the archive).
