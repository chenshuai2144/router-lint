# router-lint

一个用来检查 routers 配置的库。

支持三种常见的错误：

## 不要使用 children 改为使用 routes

```bash

error[no-use-children]: 🚨 不应该使用 children 来配置子路由, children 已经废弃，请使用 routes 来代替！
  --> .\routes.ts:33:3
   |
33 |     {
   |  ___^
34 | |     path: '/admin',
35 | |     name: 'admin',
36 | |     icon: 'crown',
37 | |     access: 'canAdmin',
38 | |     component: './Admin',
39 | |     children: [
40 | |       {
41 | |         path: '/admin/sub-page',
42 | |         name: 'sub-page',
43 | |         icon: 'smile',
44 | |         component: './Welcome',
45 | |       },
46 | |       {
47 | |         component: './404',
48 | |       },
49 | |     ],
50 | |   },
   | |___^
   |
```

## redirect 路由中应该只配置 redirect 和 path 两个属性


```bash


error[redirect-only-has-redirect-and-path]: 🚨 redirect 路由中应该只配置 redirect 和 path 两个属性！
  --> .\routes.ts:57:3
   |
57 |     {
   |  ___^
58 | |     path: '/',
59 | |     redirect: '/welcome',
60 | |     component: './404',
61 | |   },
   | |___^
   |
   
```

## path发现重复，可能会导致路径渲染错误，请检查后删除

   
   ```bash
error[redirect-only-has-redirect-and-path]: 🚨 path发现重复，可能会导致路径渲染错误，请检查后删除！
  --> .\routes.ts:9:17
   |
 9 |           routes: [
   |  _________________^
10 | |           {
11 | |             name: 'login',
12 | |             path: './login',
13 | |             component: './user/Login',
14 | |           },
15 | |           {
16 | |             name: 'login',
17 | |             path: './login',
18 | |             component: './user/Login',
19 | |           },
20 | |         ],
   | |_________^
   |
```
