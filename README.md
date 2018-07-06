# tifari
A taggable imageboard for artist reference images, self-hosted.

## Projects

| Project | Purpose |
----------|-----------
| backend | Expose methods of interacting with the database |
| backend-api | Expose a web API for interacting with the backend and serving images. |
| frontend | Expose a UX for the backend-api |
| models | Store shared object models |
| prod | An all-in-one server that serves the API, images and frontend to the user |

## Building

Requirements:
* npm
  * react scripts
* rust nightly (lowest tested working version is 2018-06-05)
  * cargo
 
The `package.sh` script will build the react frontend and the rust server that includes the API backend, the image server and the frontend server.

### For contributors

If you are looking to make changes and then test them, then I recommend running the backend-api and restarting that when needed while keeping the react frontend runnig continuously.

## License

MIT
