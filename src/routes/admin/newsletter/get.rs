use {
    actix_web::{http::header::ContentType, HttpResponse},
    actix_web_flash_messages::IncomingFlashMessages,
    std::fmt::Write,
};

pub async fn publish_issue_form(
    flash_message: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    let idempotency_key = uuid::Uuid::new_v4();
    for m in flash_message.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Publish Newsletter Issue</title>
    </head>
    <body>
        {msg_html}
        <form action="/admin/newsletter" method="post">
            <label>Title:<br>
                <input
                    type="text"
                    placeholder="Enter title"
                    name="title"
                />
            </label>
            <br>
            <label>Plain text content:<br>
                <textarea
                    placeholder="Enter the content in plain text"
                    name="text_content"
                    rows="20"
                    cols="50"
                />
            </label>
            <br>
            <label>HTML content:<br>
                <textarea
                    placeholder="Enter the content in HTML format"
                    name="html_content"
                    rows="20"
                    cols="50"
                />
            </label>
            <br>
            <input hidden type="text" name="idempotency_key" value="{idempotency_key}" />
            <button type="submit">Submit</button>
        </form>
        <p><a href="/admin/dashboard">&lt;- Back</a></p>
    </body>
</html>
    "#
        )))
}
