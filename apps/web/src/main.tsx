import {StrictMode,useState} from "react";import{createRoot}from"react-dom/client";import"./style.css";
type Settlement={id:string;name:string;population:number;treasury:number;inventory:Record<string,number>};
type World={id:string;tick:number;seed:number;settlements:Record<string,Settlement>;resource_sites:Record<string,{name:string;stocks:Record<string,number>}>;events:{tick:number;cause:string;actor:string;payload:{type:string}}[]};
function App(){
 const[w,setW]=useState<World|null>(null);const[err,setErr]=useState("");const[busy,setBusy]=useState(false);
 const api=import.meta.env.VITE_API_URL||"http://127.0.0.1:8080";
 async function request(path:string,method="GET",body?:unknown){setBusy(true);setErr("");try{const r=await fetch(api+path,{method,headers:body?{"Content-Type":"application/json"}:{},body:body?JSON.stringify(body):undefined});if(!r.ok)throw Error(await r.text());setW(await r.json());}catch(e){setErr(String(e));}finally{setBusy(false);}}
 const settlements=w?Object.values(w.settlements):[];
 return <main><header><div className="symbol">S</div><div><small>HISTORICAL WORLD SIMULATION</small><h1>SHERLOCK</h1></div><span>WORLD ENGINE / DEV</span></header>
 <section className="hero"><small>A WORLD THAT REMEMBERS</small><h2>История не задана.<br/><i>Она возникает.</i></h2><p>Цивилизация как единая система: ресурсы, люди, производство, обмен и власть. Каждый шаг оставляет причинный след.</p>{!w?<button disabled={busy} onClick={()=>request("/api/worlds","POST")}>Создать тестовый мир</button>:<button disabled={busy} onClick={()=>request("/api/worlds/"+w.id+"/advance","POST")}>Продвинуть время на один такт</button>}{err&&<pre>{err}</pre>}</section>
 {w&&<section><div className="meta">МИР {w.id.slice(0,8)} <b>ТАКТ {w.tick}</b> SEED {w.seed}</div>
 <div className="stats">{settlements.map(s=><article key={s.id}><small>{s.name.toUpperCase()}</small><strong>{s.population.toLocaleString("ru-RU")} жителей</strong><span>Казна: {s.treasury}</span><span>Еда: {s.inventory.food??0} · Дерево: {s.inventory.timber??0} · Камень: {s.inventory.stone??0}</span></article>)}</div>
 <div className="actions"><button disabled={busy} onClick={()=>request("/api/worlds/"+w.id+"/extract","POST",{site_id:"stone-quarry",settlement_id:"hill-village",good:"stone",quantity:25,actor:"player"})}>Добыть 25 камня</button><button disabled={busy} onClick={()=>request("/api/worlds/"+w.id+"/trade","POST",{seller_id:"hill-village",buyer_id:"river-town",good:"stone",quantity:3,unit_price:10,actor:"player"})}>Купить 3 камня за 30</button></div>
 <div className="site">{Object.values(w.resource_sites).map(s=><span key={s.name}>МЕСТО ДОБЫЧИ · {s.name} · Камень: {s.stocks.stone??0}</span>)}</div>
 <h3>Летопись причин</h3>{w.events.slice(-12).reverse().map((e,i)=><div className="event" key={i}><small>ТАКТ {e.tick}</small><b>{e.cause}</b><span>{e.payload.type} · {e.actor}</span></div>)}</section>}
 <footer>СОСТОЯНИЕ ПОЛУЧЕНО ИЗ API · SHERLOCK</footer></main>
}
createRoot(document.getElementById("root")!).render(<StrictMode><App/></StrictMode>);
