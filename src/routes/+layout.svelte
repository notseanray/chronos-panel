<script lang="ts">
    import "../app.css";
    import { onMount } from "svelte";
    const HOST = "http://localhost:8080"
    const DISCORD_API_URL = "https://discordapp.com/api";
    let user: any;
    onMount(async () => {
        let cookie = document.cookie.split(";").map((x) => x.split("="));
        let cookies = new Map();
        for (const [k, v] of cookie) {
            cookies.set(k, v);
        }
        console.log(cookies)

          // if only refresh token is found, then access token has expired. perform a refresh on it.
          /*
          if (cookies.get("disco_refresh_token") && !cookies.get("disco_access_token")) {
            const discord_request = await fetch(`${HOST}/api/refresh?code=${cookies.get("disco_refresh_token")}`);
            const discord_response = await discord_request.json();

            if (discord_response.disco_access_token) {
              console.log('setting discord user via refresh token..')
              const request = await fetch(`${DISCORD_API_URL}/users/@me`, {
                headers: { 'Authorization': `Bearer ${discord_response.disco_access_token}` }
              });

              // returns a discord user if JWT was valid
              const response = await request.json();

              if (response.id) {
                return {
                  user: {
                    // only include properties needed client-side —
                    // exclude anything else attached to the user
                    // like access tokens etc
                    ...response
                  }
                }
              }
            }
          }
          */
      if (cookies.get("disco_access_token")) {
        console.log('setting discord user via access token..')
        const request = await fetch(`${DISCORD_API_URL}/users/@me`, {
          headers: { 'Authorization': `Bearer ${cookies.get("disco_access_token")}`}
        });

        // returns a discord user if JWT was valid
        const response = await request.json();

        if (response.id) {
            user = {
              // only include properties needed client-side —
              // exclude anything else attached to the user
              // like access tokens etc
              ...response
            }
          }
        } else {
          user = false;
        }
      })
  console.log('user', user)
</script>

<main class="bg-[#2E3440] h-full min-h-screen text-white">
	<div class="bg-[#3B4252]">
		<div class="flex w-full p-3 h-14">
		  <div class="text-white text-xl">Chronos Panel</div>
		  <div class="float-right flex ml-auto order-2 mr-9">
			{#if !user}
			  <button class="btn btn-xs btn-active btn-neutral hover:shadow-2xl" on:click={async () => {
				  console.log("test")
				  await fetch("http://localhost:8080/api/auth")
			  }}>Authenticate with Discord</button>
			{:else}
			  <img class="rounded-full h-8" alt="{user.userName}#{user.discriminator} avatar" src="https://cdn.discordapp.com/avatars/{user.id}/{user.avatar}.png">
			  <div class="flex-wrap p-1 text-sm align-middle">
				  <div>{user.global_name}</div>
			  </div>
			  <div class="flex-wrap p-1 text-sm">
				  <button class="btn btn-xs btn-active btn-neutral hover:shadow-2xl" on:click={async () => {
					  await fetch("http://localhost:8080/api/signout")
				  }}>Sign Out</button>
			  </div>
			{/if}
		  </div>
		</div>
	</div>
	{#if user}
		<div class="p-5">
			<slot />
		</div>
	{/if}
</main>
