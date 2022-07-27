<script>
  import CnPhrase from '../components/CnPhrase.svelte'
</script>

<body>
    <header class="page-header page-header-light bg-white">
        <div class="page-header-content">
            <div class="container">
                <h1>About</h1>
                <p>
                    DuGuo (<CnPhrase phrase="读国" />)
                    originated from Georgia Tech's OMSCS EdTech class (Spring 2019). It was adopted from the Chinese Reading Machine (<CnPhrase phrase="中文读机" />) 
                    which was a Flask-based application (and an incredibly helpful starting point).
                </p>
                <p>
                    When I picked up this project, I really knew nothing about web dev. Since then, with help from many people and online resources, I've learned a ton though still don't have much experience in web dev. Most of this site uses vanilla JS and DOM-manipulation and will likely stay that way for a while.
                </p>
                <p>
                    Regardless, there are many interesting technologies and concepts that I find exciting (e.g. WASM) and will try to incorporate them into this application when possible! I really am a backend engineer though. That dabbles in frontend. Please enjoy (and/or suggest updates for) any frontend hacks you stumble across.
                </p>
                <h2>Mission</h2>
                <p>
                    Provide accessible and tailored Chinese education to improve Chinese literacy and cross-cultural connection, and have some fun with it!
                </p>
                <h2>Acknowledgements</h2>
                <p>
                    Marked in the code/docs whenever it comes-up, though as with any software project, this relies heavily on many software libraries and projects that are charitably developed and maintained, and I greatly appreciate the many people behind these efforts. Also thank you to AWS for a lovely year of free-tier and to GCP for my next year.
                </p>
                <p>Last Updated: 12/22/21 by Eric Pan</p>
            </div>
        </div>
    </header>
</body>