/// Load DataTables
$(document).ready(() => { 
    $('#doc-table').DataTable();
    $('#vocab-table').DataTable();
});

/// Remove characters that cause display issues
let escape_formatting = (s) => {
    s = s.replace(/\r/g, '');
    s = s.replace(/\n/g, '\\n');
    if (s.includes(',')) {
        return `"${s}"`; // encase in double-quotes
    } else {
        return s;
    }
}

/// Performs csv download using the specified <a> tag id
let perform_download = (csv, anchor_id, filename) => {
    let e = document.getElementById(anchor_id);
    e.href = 'data:text/csv;charset=utf-8,' + encodeURI(csv);
    e.download = filename;
    e.click();
}

/// Formats GET request to /api/docs-to-csv and downloads as .csv
let download_doc_table_as_csv = (anchor_id) => {
    let xhr = new XMLHttpRequest();
    xhr.open("GET", "/api/docs-to-csv");
    xhr.onreadystatechange = () => {
        if (xhr.readyState == 4 && xhr.status == 200) {
            let csv_header = 'title,body,source,created_on\n';
            let csv_body = '';
            let json = JSON.parse(xhr.response);
            for (i = 0; i < json.title.length; i++) {
                let title = escape_formatting(json.title[i]);
                let body = escape_formatting(json.body[i]);
                let source = escape_formatting(json.source[i]);
                let created_on = escape_formatting(json.created_on[i]);
                csv_body += `${title},${body},${source},${created_on}\n`;
            }
            let csv = csv_header + csv_body;
            perform_download(csv, anchor_id, 'duguo-documents.csv');
        }
    }
    xhr.send();
}

/// Formats GET request to /api/vocab-to-csv and downloads as .csv
let download_vocab_table_as_csv = (anchor_id) => {
    let xhr = new XMLHttpRequest();
    xhr.open("GET", "/api/vocab-to-csv");
    xhr.onreadystatechange = () => {
        if (xhr.readyState == 4 && xhr.status == 200) {
            let csv_header = 'phrase,phonetics,definition,radical_map,from_doc_title,saved_on\n';
            let csv_body = '';
            let json = JSON.parse(xhr.response);
            for (i = 0; i < json.phrase.length; i++) {
                let phrase = escape_formatting(json.phrase[i]);
                let phonetics = escape_formatting(json.phrase_phonetics[i]);
                let def = escape_formatting(json.def[i]);
                let radical_map = escape_formatting(json.radical_map[i]);
                let from_doc = escape_formatting(json.from_doc_title[i]);
                let created_on = escape_formatting(json.created_on[i]);
                csv_body += `${phrase},${phonetics},${def},${radical_map},${from_doc},${created_on}\n`;
            }
            let csv = csv_header + csv_body;
            perform_download(csv, anchor_id, 'duguo-vocab.csv');
        }
    }
    xhr.send();
}