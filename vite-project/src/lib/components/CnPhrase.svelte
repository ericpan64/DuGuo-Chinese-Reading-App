<script>
    export let phrase;

    // Lookup the phrase via WASM
    let uid = "读国du2guo2";
    let definition = "1. An open-source application to facilitate Chinese reading. Stems from 读 (to read) and 国 (country)<br>2. An extended implementation of the 中文读机 which was originally developed by Martin Kess from the OMSCS Education Technology class";
    let title_html = "读国 [du2 guo2] <a role=&quot;button&quot; href=&quot;#~读国&quot;><img src=&quot;/static/img/volume-up-fill.svg&quot;></img></a> <a role=&quot;button&quot; href=&quot;#读国du2guo2&quot;><img src=&quot;/static/img/download.svg&quot;></img></a>";
</script>

<span tabindex="0" data-bs-toggle="popover" data-bs-html="true"
    class={uid}
    data-bs-content={definition} 
    title={title_html} >
    {phrase}
</span>
