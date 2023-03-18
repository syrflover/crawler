export const gg = async (
    content_id: number,
    code_number: number
): Promise<{ m: number; b: string }> => {
    const res = await fetch("https://ltn.hitomi.la/gg.js", {
        headers: {
            Referer: `https://hitomi.la/reader/${content_id}.html`,
        },
    });

    const gg_js = (await res.text()).replace(/return.{0,};/g, "return o;");

    // console.log(gg_js);

    const a = eval(`
    let gg = {};

    const $ = () => ({
        attr: () => {},
    });

    const window = {};

    const document = {
        title: "Hitomi.la",
        location: {
            hostname: "hitomi.la"
        },
        documentElement: {
            clientHeight: 400
        },
    };

    ${gg_js}
    
    const ret = () => ({
        m: gg.m(${code_number}),
        b: gg.b.replaceAll('/', ''),
    })

    ret()
    `);

    // console.log(typeof a, a);

    return a;
};

// gg(2277336, 2828);
