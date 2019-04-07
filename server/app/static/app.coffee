# Utility functions for making the front-end come alive!

window.speak = (word) ->
    utterance = new SpeechSynthesisUtterance word
    utterance.lang = 'zh-CN'
    utterance.rate = 0.8  # slow it down a bit for learners
    window.speechSynthesis.speak utterance

window.saveVocab = (e, word) ->
    $.ajax
        type: 'POST'
        url: '/api/vocab/add'
        data:
            phrase: word
        success: (data) ->
            # TODO: Change button class/icon.
            $("[data-word='" + word + "']").removeClass('not-in-vocab').addClass('in-vocab')
            $(e).remove()
        error: (req, status, err) ->
            alert req.responseJSON['error']

window.deleteVocab = (word) ->
    m = $('#myModal')
    m.find('#vocab-word').text(word)
    d = $('#delete', m)
    d.click () ->
        $.ajax
            type: 'POST'
            url: '/api/vocab/delete'
            data:
                phrase: word
            success: (data) ->
                $("[data-word='" + word + "']").remove()
                m.modal('hide')
            error: (req, status, err) ->
                alert req.responseJSON['error']
    m.modal()

window.deleteDocument = (document_id) ->
    m = $('#myModal')
    d = $('#delete', m)
    d.click () ->
        $.ajax
            type: 'POST'
            url: '/delete/' + document_id
            success: () ->
                $('#document_' + document_id).remove()
                m.modal('hide')
            error: (req, status, err) ->
                alert 'ERROR', req, status, err
    m.modal()


$('.word').bind 'click', () ->
    # We use the first click to load the content for the tooltip
    $(this).unbind 'click'
    # Pull the word out of the element
    word = $(this).attr 'data-word'
    in_vocab = $(this).hasClass('in-vocab')
    this_ = this
    $.getJSON '/api/define?word=' + word, (data) ->
        # Now that we have the data, create the pop-over
        title =
            '<b>' + word + '</b>' + '&nbsp;' +
            '<span class="pull-right">'

        if not in_vocab
            title += '<button class="btn btn-xs btn-success" onclick="javascript:window.saveVocab(this, \'' + word + '\');">' +
                        '<span class="glyphicon glyphicon-plus" id="save" ></span>' +
                    '</button>'

        title += '&nbsp;' +
            '<button class="btn btn-xs btn-info" onclick="javascript:window.speak(\'' + word + '\');">' +
                '<span class="glyphicon glyphicon-volume-up"></span>' +
            '</button>' +
            '</span>'

        defs = data['definitions']
        content = ''
        for def in defs
            content += '<b>' + def['pinyin'] + '</b>'
            content += '<ul>'
            for d in def['definitions']
                content += '<li>' + d + '</li>'
            content += '</ul>'

        $(this_).popover
            title: title
            content: content
            html: true
            placement: 'auto top'
            trigger: 'focus click'

        $(this_).click()


# Call it once with no args to initialize TTS system - otherwise first play
# will take a few seconds
window.speak ''
