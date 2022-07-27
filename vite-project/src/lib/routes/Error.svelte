<script>
    export let error_number;
    import Duey from '../components/Duey.svelte'
</script>

<header class="page-header page-header-light bg-white">
    <div class="page-header-content">
        <div class="container">
            <h1>HTTP Error {{error_number}}</h1>
            {#if error_number == 500}
            <Duey extra emote="worried" />
            <p>There was an error on our end, sorry about that!</p>
            {:else}
                <img width="200em" id="duey" src="/static/img/duey/duey_extra_confused.png"
                alt="Duey, the DuGuo mascot!"/>
                <p>The page you're looking for doesn't seem to exist!</p>
            {/if}
            <p>Submit feedback <a href="/feedback">here</a> and/or open a GitHub issue <a target="_blank" href="https://github.com/ericpan64/DuGuo-Chinese-Reading-App">here</a>.</p>
        </div>
    </div>
</header>