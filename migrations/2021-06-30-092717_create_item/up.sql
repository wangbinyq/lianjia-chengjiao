-- Your SQL goes here

create table if not exists items (
  id integer not null primary key,

  qu text not null,
  zheng text not null,
  url text not null unique,

  name text not null,
  huxing text not null,
  floor text not null,
  square text not null,
  structs text not null,
  inner_square text not null,
  build_type text not null,
  direction text not null,
  build_year text not null,
  build_decorate text not null,
  build_struct text not null,
  gongnuan text not null,
  tihubi text not null,
  dianti text not null,

  quansu text not null,
  guapai_time text not null,
  yongtu text not null,
  nianxian text not null,
  fangquan text not null,

  guapai_price text not null,
  chengjiao_zhouqi text not null,
  tiaojia text not null,
  chengjiao_price text not null,
  danjia text not null
);