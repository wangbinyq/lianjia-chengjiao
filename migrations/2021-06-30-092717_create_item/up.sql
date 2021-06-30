-- Your SQL goes here

create table if not exists items (
  id integer not null primary key,

  qu text not null,
  zheng text not null,
  url text not null unique,

  name text not null,
  huxing text not null,
  floor text not null,
  floor_number int not null,
  square real not null,
  structs text not null,
  inner_square real not null,
  build_type text not null,
  direction text not null,
  build_year int not null,
  build_decorate text not null,
  build_struct text not null,
  gongnuan text not null,
  tihubi text not null,
  dianti text not null,

  quansu text not null,
  guapai_day date ,
  yongtu text not null,
  nianxian text not null,
  fangquan text not null,

  guapai_price int not null,
  chengjiao_zhouqi int not null,
  tiaojia int not null,
  chengjiao_price int not null,
  chengjiao_day date,
  danjia int not null
);