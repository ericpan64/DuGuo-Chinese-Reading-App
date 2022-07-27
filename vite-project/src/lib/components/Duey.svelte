<script>
    export let emote;
    export let extra = true;
    export let width = "200em";

    let dir = "/static/img/duey"
    let img = "";
    if (extra) {
        img = `${dir}/duey_extra_${emote}.png`
    } else {
        img = `${dir}/duey_${emote}.png`
    }
</script> 

<img {width} id="duey" src={img} alt="Duey, the DuGuo mascot! ({emote})"/>