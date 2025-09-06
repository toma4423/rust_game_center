<script lang="ts">
    import { onMount } from 'svelte';

    let socket: WebSocket;
    let displayName = '';
    let roomId = '';
    let roomState: any = null;
    let serverMessages: string[] = [];
    let myPlayerId: string | null = null; // 自分のIDを保持

    // ゲーム関連
    let selectedGameId = '';
    const availableGames = [{ id: 'quiz', title: '早押しクイズ' }]; // 本来はサーバーから取得

    onMount(() => {
        socket = new WebSocket('ws://127.0.0.1:3000/ws');

        socket.onopen = () => {
            serverMessages = [...serverMessages, 'Connected to server'];
        };

        socket.onmessage = (event) => {
            const message = JSON.parse(event.data);
            serverMessages = [...serverMessages, `RECV: ${event.data}`];

            if (message.type === 'room_update') {
                roomState = message.room;
                // 最初のroom_updateで自分のIDを特定（簡易的）
                if (!myPlayerId) {
                    myPlayerId = roomState.players[roomState.players.length - 1].id;
                }
            } else if (message.type === 'game_state' || message.type === 'broadcast_event') {
                // ゲームの状態やイベントも表示
                alert(`EVENT: ${message.event || message.phase} - ${JSON.stringify(message.data)}`);
            } else if (message.type === 'error') {
                alert(`Error: ${message.message}`);
            }
        };

        socket.onclose = () => {
            serverMessages = [...serverMessages, 'Disconnected from server'];
            myPlayerId = null;
            roomState = null;
        };

        return () => {
            socket.close();
        };
    });

    function createRoom() {
        if (!displayName) return alert('Please enter a display name.');
        const msg = { type: 'create_room', display_name: displayName };
        socket.send(JSON.stringify(msg));
    }

    function joinRoom() {
        if (!displayName || !roomId) return alert('Please enter a display name and room ID.');
        const msg = { type: 'join_room', room_id: roomId, display_name: displayName };
        socket.send(JSON.stringify(msg));
    }

    function selectGame() {
        if (!selectedGameId) return;
        const msg = { type: 'select_game', game_id: selectedGameId };
        socket.send(JSON.stringify(msg));
    }

    function startGame() {
        const msg = { type: 'start_game' };
        socket.send(JSON.stringify(msg));
    }

    function sendFirstPress() {
        const msg = { type: 'realtime_action', action: 'first_press' };
        socket.send(JSON.stringify(msg));
    }

</script>

<main class="container">
    <h1>Rust GameCenter</h1>

    {#if !roomState}
        <div class="grid">
            <div>
                <label for="displayName">Display Name</label>
                <input type="text" id="displayName" bind:value={displayName} />
            </div>
            <div>
                <label for="roomId">Room ID</label>
                <input type="text" id="roomId" bind:value={roomId} placeholder="12345"/>
            </div>
        </div>
        <div class="grid">
            <button on:click={createRoom}>Create Room</button>
            <button on:click={joinRoom}>Join Room</button>
        </div>
    {:else}
        <article>
            <header>Room: {roomState.id} {#if roomState.state === 'InGame'}<strong>(In Game)</strong>{/if}</header>
            <p><strong>Host:</strong> {roomState.players.find(p => p.id === roomState.host_id)?.display_name}</p>
            <p><strong>Players:</strong></p>
            <ul>
                {#each roomState.players as player}
                    <li>{player.display_name} {#if player.id === myPlayerId}<strong>(You)</strong>{/if}</li>
                {/each}
            </ul>
            <p><strong>Selected Game:</strong> {roomState.selected_game?.title || 'None'}</p>
            <p><strong>Game Phase:</strong> {roomState.realtime_state.game_phase}</p>
            {#if roomState.realtime_state.first_press_winner}
                 <p><strong>First Press Winner:</strong> {roomState.players.find(p => p.id === roomState.realtime_state.first_press_winner)?.display_name}</p>
            {/if}
            
            {#if roomState.state !== 'InGame' && roomState.host_id === myPlayerId}
                <section>
                    <h3>Host Controls</h3>
                     <div class="grid">
                        <select bind:value={selectedGameId}>
                            <option value="">Select a game</option>
                            {#each availableGames as game}
                                <option value={game.id}>{game.title}</option>
                            {/each}
                        </select>
                        <button on:click={selectGame} disabled={!selectedGameId}>Select Game</button>
                    </div>
                    <button on:click={startGame} disabled={!roomState.selected_game}>Start Game</button>
                </section>
            {/if}

            {#if roomState.state === 'InGame'}
            <footer>
                <button on:click={sendFirstPress}>First Press!</button>
            </footer>
            {/if}
        </article>
    {/if}

    <article>
        <header>Server Messages</header>
        <pre><code>{#each serverMessages as msg}{msg}{"\n"}{/each}</code></pre>
    </article>

</main>

<style>
    .container {
        max-width: 800px;
        margin: 0 auto;
        padding: 2rem;
    }
    .grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
        margin-bottom: 1rem;
    }
    article {
        margin-top: 2rem;
    }
    pre {
        background: #f4f4f4;
        padding: 1rem;
        max-height: 200px;
        overflow-y: auto;
    }
    section {
        border-top: 1px solid #ccc;
        margin-top: 1rem;
        padding-top: 1rem;
    }
</style>
