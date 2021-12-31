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
    xhr.open("GET", "/api/get-all-user-items");
    xhr.onreadystatechange = () => {
        if (xhr.readyState == 4 && xhr.status == 200) {
            let csv_header = 'title,body,source,created_on\n';
            let csv_body = '';
            let json = JSON.parse(xhr.response);
            let json_list = json.doc_list;
            for (i = 0; i < json_list.length; i++) {
                let title = escape_formatting(json_list[i].title);
                let body = escape_formatting(json_list[i].body);
                let source = escape_formatting(json_list[i].source);
                let created_on = escape_formatting(json_list[i].created_on);
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
    xhr.open("GET", "/api/get-all-user-items");
    xhr.onreadystatechange = () => {
        if (xhr.readyState == 4 && xhr.status == 200) {
            let csv_header = 'phrase,phonetics,definition,radical_map,from_doc_title,saved_on,phrase_html\n';
            let csv_body = '';
            let json = JSON.parse(xhr.response);
            let json_list = json.vocab_list;
            for (i = 0; i < json_list.length; i++) {
                let phrase = escape_formatting(json_list[i].phrase);
                let phonetics = escape_formatting(json_list[i].phrase_phonetics);
                let def = escape_formatting(json_list[i].def);
                let radical_map = escape_formatting(json_list[i].radical_map);
                let from_doc = escape_formatting(json_list[i].from_doc_title);
                let created_on = escape_formatting(json_list[i].created_on);
                let phrase_html = escape_formatting(json_list[i].phrase_html);
                csv_body += `${phrase},${phonetics},${def},${radical_map},${from_doc},${created_on},${phrase_html}\n`;
            }
            let csv = csv_header + csv_body;
            perform_download(csv, anchor_id, 'duguo-vocab.csv');
        }
    }
    xhr.send();
}